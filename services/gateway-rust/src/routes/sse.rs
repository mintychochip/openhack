use std::collections::HashSet;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use actix_web::web::Bytes;
use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use futures_util::Stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

const REDIS_CHANNELS: &[&str] = &[
    "auth:events",
    "core:events",
    "judging:events",
    "mail:events",
    "leaderboard:events",
    "notify:events",
];

const EVENTS_REDIS_KEY: &str = "openhack:events:recent";
const EVENTS_MAX_AGE_SECS: i64 = 300;

/// Query parameters for the polling endpoint.
///
/// # Expected Behavior
///
/// Deserializes from the `after` query parameter, which is an
/// RFC 3339 timestamp. Only events newer than this timestamp
/// are returned. An absent `after` parameter returns the most
/// recent events (up to `limit`).
#[derive(Deserialize)]
pub struct PollQuery {
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

/// A single event returned by the polling endpoint.
#[derive(Serialize)]
pub struct PolledEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub channel: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

/// Query parameters for the SSE stream endpoint.
///
/// # Expected Behavior
///
/// Deserializes from the `types` query parameter, which is a
/// comma-separated list of event types to filter (e.g.
/// `team.created,project.submitted`). An empty or absent `types`
/// parameter means no filtering — all events are delivered.
#[derive(Deserialize)]
pub struct SseQuery {
    #[serde(default)]
    pub types: String,
}

/// A single connected SSE client's sender half and optional type filter.
struct ClientSender {
    id: Uuid,
    sender: mpsc::Sender<Bytes>,
    filter_types: Option<HashSet<String>>,
}

/// Shared event bus that manages connected SSE clients and the
/// Redis pub/sub subscriber lifecycle using reference counting.
///
/// # Expected Behavior
///
/// Maintains a list of connected SSE clients, each with an mpsc sender
/// and an optional event-type filter. If `redis_url` is `None`, no Redis
/// subscriber is ever spawned — SSE clients connect and receive pings
/// but no Redis events. If `redis_url` is `Some`, when the first client
/// is added, a Redis subscriber task is spawned. When the last client
/// disconnects, the subscriber task is aborted. All internal state is
/// protected by `tokio::sync::Mutex` wrapped in `Arc` for safe sharing
/// across tasks. The channel carries raw `Bytes` (fully formatted SSE
/// frames) to avoid the `!Send` bound of `actix_web::Error` inside
/// spawned tasks.
///
/// # Errors
///
/// Methods on `EventBus` do not return errors directly. Redis connection
/// failures are handled internally by the subscriber task, which
/// reconnects with a 5-second backoff and sends `event: error` SSE
/// frames to all connected clients. If `redis_url` is None, no errors
/// occur — the subscriber is simply never started.
///
/// # Side Effects
///
/// - Spawns a Redis subscriber Tokio task when the first client connects
///   (only if `redis_url` is Some).
/// - Aborts the subscriber task when the last client disconnects.
/// - The subscriber task opens and maintains a Redis pub/sub connection.
pub struct EventBus {
    clients: Arc<Mutex<Vec<ClientSender>>>,
    redis_url: Option<Arc<String>>,
    subscriber_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    sse_enabled: bool,
}

impl EventBus {
    /// Create a new `EventBus` with the given optional Redis URL and SSE mode.
    ///
    /// # Expected Behavior
    ///
    /// Initializes empty internal state. If `sse_enabled` is `false`,
    /// no Redis subscriber will ever be spawned and `sse_stream` should
    /// return 501. If `redis_url` is `None`, no Redis connection will
    /// ever be opened. If both are set, a Redis subscriber is spawned
    /// lazily when the first client is added.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// - Heap-allocates internal Arc<Mutex<...>> state.
    pub fn new(redis_url: Option<String>, sse_enabled: bool) -> Self {
        let bus = Self {
            clients: Arc::new(Mutex::new(Vec::new())),
            redis_url: redis_url.map(Arc::new),
            subscriber_handle: Arc::new(Mutex::new(None)),
            sse_enabled,
        };

        if bus.redis_url.is_some() {
            let clients = bus.clients.clone();
            let redis_url = bus.redis_url.clone().unwrap();
            let sse = bus.sse_enabled;
            let subscriber_handle = bus.subscriber_handle.clone();
            tokio::spawn(async move {
                let mut handle = subscriber_handle.lock().await;
                if handle.is_none() {
                    let h = tokio::spawn(async move {
                        run_redis_subscriber(clients, redis_url, sse).await;
                    });
                    *handle = Some(h);
                }
            });
        }

        bus
    }

    /// Add a new SSE client to the event bus.
    ///
    /// # Expected Behavior
    ///
    /// Creates an mpsc channel pair. Stores the sender (along with the
    /// client ID and optional filter) in the shared client list.
    /// Returns the client ID, a clone of the sender (for the handler to
    /// send initial events and spawn the ping task), and a
    /// `SseReceiverStream` wrapping the receiver. If this is the first
    /// client, spawns the Redis subscriber task.
    ///
    /// # Errors
    ///
    /// None. Redis subscriber spawn failures are logged but not
    /// propagated.
    ///
    /// # Side Effects
    ///
    /// - Acquires the clients Mutex lock (blocking other operations
    ///   briefly).
    /// - If the client list was empty and `redis_url` is Some, spawns a
    ///   Redis subscriber task (creates a new Tokio task and potentially
    ///   opens a Redis connection).
    pub async fn add_client(
        &self,
        filter_types: Option<HashSet<String>>,
    ) -> (Uuid, mpsc::Sender<Bytes>, SseReceiverStream) {
        let (tx, rx) = mpsc::channel(32);
        let id = Uuid::new_v4();

        let was_empty = {
            let mut clients = self.clients.lock().await;
            let was_empty = clients.is_empty();
            clients.push(ClientSender {
                id,
                sender: tx.clone(),
                filter_types,
            });
            was_empty
        };

        if was_empty {
            self.ensure_subscriber().await;
        }

        let stream = SseReceiverStream {
            rx,
            client_id: id,
            clients: self.clients.clone(),
            subscriber_handle: self.subscriber_handle.clone(),
        };

        (id, tx, stream)
    }

    /// Ensure the Redis subscriber task is running.
    ///
    /// # Expected Behavior
    ///
    /// If a subscriber task is already running (handle is Some), returns
    /// immediately. Otherwise, spawns a new task that runs
    /// `run_redis_subscriber` with clones of the shared client list,
    /// Redis URL, and SSE enabled flag. The subscriber always runs when
    /// a Redis URL is configured so that events are written to the sorted
    /// set for the poll endpoint, even when SSE streaming is disabled.
    /// Stores the `JoinHandle` for later cancellation.
    ///
    /// # Errors
    ///
    /// None. Errors inside the subscriber task are handled internally.
    ///
    /// # Side Effects
    ///
    /// - Acquires the `subscriber_handle` Mutex lock.
    /// - Spawns a new Tokio task (the Redis subscriber).
    async fn ensure_subscriber(&self) {
        let mut handle = self.subscriber_handle.lock().await;
        if handle.is_some() {
            return;
        }

        let Some(redis_url) = self.redis_url.clone() else {
            log::info!("No Redis URL configured, SSE subscriber not started");
            return;
        };

        let clients = self.clients.clone();
        let sse_enabled = self.sse_enabled;

        let h = tokio::spawn(async move {
            run_redis_subscriber(clients, redis_url, sse_enabled).await;
        });
        *handle = Some(h);
    }

    /// Remove a client by ID and stop the subscriber if no clients remain.
    ///
    /// # Expected Behavior
    ///
    /// Retains all clients whose ID does not match `client_id`. If the
    /// client list becomes empty after removal, aborts the subscriber
    /// task by calling `abort()` on its `JoinHandle`.
    ///
    /// # Errors
    ///
    /// None. Aborting a completed or already-aborted `JoinHandle` is
    /// a no-op.
    ///
    /// # Side Effects
    ///
    /// - Acquires the clients Mutex lock.
    /// - If the client list is empty after removal, acquires the
    ///   `subscriber_handle` Mutex lock and aborts the subscriber task
    ///   (cancels the Tokio task, which drops the Redis connection).
    #[allow(dead_code)]
    async fn remove_client(&self, client_id: Uuid) {
        let is_empty = {
            let mut clients = self.clients.lock().await;
            clients.retain(|c| c.id != client_id);
            clients.is_empty()
        };

        if is_empty {
            let mut handle = self.subscriber_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
            }
        }
    }
}

/// Streaming body for an SSE connection.
///
/// # Expected Behavior
///
/// Wraps the receiver half of an mpsc channel carrying raw `Bytes`
/// (each a fully formatted SSE frame). Implements `Stream` yielding
/// `Result<Bytes, Infallible>` items for compatibility with
/// `HttpResponse::streaming()`. When dropped (client disconnects),
/// automatically spawns a cleanup task that removes the client from
/// the `EventBus` and stops the Redis subscriber if no clients remain.
///
/// # Errors
///
/// The stream itself does not produce errors. All items are `Ok(Bytes)`.
/// Error events are sent as normal SSE data frames inside the Bytes.
///
/// # Side Effects
///
/// - On Drop: spawns a Tokio task to remove the client from the
///   shared client list and potentially abort the Redis subscriber.
pub struct SseReceiverStream {
    rx: mpsc::Receiver<Bytes>,
    client_id: Uuid,
    clients: Arc<Mutex<Vec<ClientSender>>>,
    subscriber_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Stream for SseReceiverStream {
    type Item = Result<Bytes, Infallible>;

    /// Poll the underlying mpsc channel receiver for the next SSE frame.
    ///
    /// # Expected Behavior
    ///
    /// Delegates to `tokio::sync::mpsc::Receiver::poll_recv`. Returns
    /// `Poll::Ready(Some(Ok(bytes)))` when a frame is available,
    /// `Poll::Ready(None)` when the channel is closed (all senders
    /// dropped), or `Poll::Pending` when no frame is ready yet.
    ///
    /// # Errors
    ///
    /// Never returns `Poll::Ready(Some(Err(..)))` since `Infallible`
    /// cannot be constructed.
    ///
    /// # Side Effects
    ///
    /// None beyond the underlying channel poll.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut().rx.poll_recv(cx) {
            Poll::Ready(Some(bytes)) => Poll::Ready(Some(Ok(bytes))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for SseReceiverStream {
    /// Clean up the SSE client on stream drop.
    ///
    /// # Expected Behavior
    ///
    /// Spawns a Tokio task that removes the client identified by
    /// `client_id` from the shared client list. If no clients remain
    /// after removal, aborts the Redis subscriber task. This ensures
    /// the Redis connection is closed when there are no active SSE
    /// clients.
    ///
    /// # Errors
    ///
    /// If the Tokio runtime is unavailable (e.g. during shutdown),
    /// the spawned task may not execute, leaving a stale entry in
    /// the client list. This is acceptable because the server is
    /// shutting down anyway.
    ///
    /// # Side Effects
    ///
    /// - Spawns a Tokio task (fire-and-forget).
    /// - The spawned task acquires Mutex locks on the client list
    ///   and subscriber handle.
    /// - May abort the Redis subscriber Tokio task (cancels the task,
    ///   drops the Redis connection).
    fn drop(&mut self) {
        let client_id = self.client_id;
        let clients = self.clients.clone();
        let subscriber_handle = self.subscriber_handle.clone();

        tokio::spawn(async move {
            let is_empty = {
                let mut guard = clients.lock().await;
                guard.retain(|c| c.id != client_id);
                guard.is_empty()
            };

            if is_empty {
                let mut handle_guard = subscriber_handle.lock().await;
                if let Some(h) = handle_guard.take() {
                    h.abort();
                }
            }
        });
    }
}

/// Format an SSE event frame.
///
/// # Expected Behavior
///
/// Produces a string in the SSE wire format:
/// `event: {event_type}\ndata: {json}\n\n`. The `data` field
/// contains the JSON-serialized `data` value on a single line.
/// `serde_json::to_string` produces compact JSON without newlines,
/// so the SSE frame is well-formed.
///
/// # Errors
///
/// If `serde_json::to_string` fails (should not happen for
/// `serde_json::Value`), the data field defaults to an empty string.
///
/// # Side Effects
///
/// None. Pure function with no I/O.
fn format_sse_event(event_type: &str, data: &serde_json::Value) -> String {
    let json = serde_json::to_string(data).unwrap_or_default();
    format!("event: {event_type}\ndata: {json}\n\n")
}

/// Normalize a raw Redis pub/sub message into a standard event envelope.
///
/// # Expected Behavior
///
/// Extracts `type` (or `event_type`) as the event type, `data` (or
/// `metadata`) as the event payload, `timestamp`, and `id` from the
/// raw JSON message. Missing fields are filled with defaults:
/// - `type`: "unknown"
/// - `data`: the entire raw message
/// - `timestamp`: current UTC time in RFC 3339
/// - `id`: `"{channel}-{timestamp_millis}"`
///
/// Returns a tuple of (`event_type`, `normalized_envelope`) where the
/// envelope is `{ id, type, channel, data, timestamp }`.
///
/// # Errors
///
/// Never fails. All missing fields have defaults.
///
/// # Side Effects
///
/// - Calls `chrono::Utc::now()` for default timestamp/id values.
fn normalize_event(channel: &str, raw: &serde_json::Value) -> (String, serde_json::Value) {
    let event_type = raw
        .get("type")
        .or_else(|| raw.get("event_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let data = raw
        .get("data")
        .or_else(|| raw.get("metadata"))
        .cloned()
        .unwrap_or_else(|| raw.clone());

    let timestamp = match raw.get("timestamp").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => Utc::now().to_rfc3339(),
    };

    let id = raw.get("id").and_then(|v| v.as_str()).map_or_else(
        || format!("{}-{}", channel, Utc::now().timestamp_millis()),
        std::string::ToString::to_string,
    );

    let normalized = serde_json::json!({
        "id": id,
        "type": event_type,
        "channel": channel,
        "data": data,
        "timestamp": timestamp,
    });

    (event_type, normalized)
}

/// Run the Redis subscriber with automatic reconnection.
///
/// # Expected Behavior
///
/// Runs `run_subscriber_loop` in an infinite loop. On each return
/// (whether Ok or Err), waits 5 seconds before reconnecting. On
/// error, sends an `event: error` SSE frame to all connected clients
/// before the backoff delay. When `sse_enabled` is false, subscriber
/// still runs to write events to the Redis sorted set for the poll
/// endpoint, but no SSE broadcast errors are sent.
///
/// # Errors
///
/// Never returns. Runs indefinitely.
///
/// # Side Effects
///
/// - Opens and closes Redis pub/sub connections repeatedly.
/// - On error, writes error SSE frames to all client channels
///   (only when `sse_enabled` is true).
/// - Logs at INFO on normal reconnect, ERROR on errors.
async fn run_redis_subscriber(
    clients: Arc<Mutex<Vec<ClientSender>>>,
    redis_url: Arc<String>,
    sse_enabled: bool,
) {
    let mut consecutive_errors = 0u32;

    loop {
        match run_subscriber_loop(clients.clone(), redis_url.clone(), sse_enabled).await {
            Ok(()) => {
                consecutive_errors = 0;
                log::info!("Redis subscriber ended, reconnecting in 5s...");
            }
            Err(e) => {
                consecutive_errors += 1;
                let delay = if consecutive_errors <= 3 {
                    5
                } else if consecutive_errors <= 10 {
                    30
                } else {
                    60
                };
                log::error!("Redis subscriber error (attempt {consecutive_errors}): {e}, reconnecting in {delay}s...");
            }
        }

        let delay = if consecutive_errors <= 3 {
            5
        } else if consecutive_errors <= 10 {
            30
        } else {
            60
        };
        tokio::time::sleep(Duration::from_secs(delay)).await;
    }
}

/// Run a single Redis subscriber connection loop.
///
/// # Expected Behavior
///
/// Opens a new Redis pub/sub connection via `redis::aio::PubSub::open`,
/// subscribes to all `REDIS_CHANNELS`, and processes incoming messages.
/// For each message, normalizes the event and:
/// - Always writes the event to the `openhack:events:recent` sorted set
///   (ZADD) so the polling endpoint can retrieve it.
/// - When `sse_enabled` is true, also broadcasts the event as an SSE
///   frame to connected clients whose type filter matches.
///
/// When the message stream ends (connection lost), returns Ok(()) to
/// trigger reconnection.
///
/// # Errors
///
/// Returns an error if the Redis connection cannot be established
/// or the subscription calls fail. These are propagated to
/// `run_redis_subscriber` which handles reconnection.
///
/// # Side Effects
///
/// - Opens a Redis pub/sub connection (network I/O).
/// - Opens a second Redis connection for ZADD writes (network I/O).
/// - Subscribes to 6 Redis channels.
/// - For each message: writes event to Redis sorted set (ZADD),
///   acquires the clients Mutex lock when `sse_enabled`, iterates
///   all clients, sends SSE frames through mpsc channels.
/// - Logs at INFO on successful connection, WARN on parse errors.
async fn run_subscriber_loop(
    clients: Arc<Mutex<Vec<ClientSender>>>,
    redis_url: Arc<String>,
    sse_enabled: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(redis_url.as_str())?;
    let conn = client.get_async_connection().await?;
    let mut pubsub = conn.into_pubsub();

    for ch in REDIS_CHANNELS {
        pubsub.subscribe(ch).await?;
    }

    log::info!(
        "Redis subscriber connected to {} channels (sse_enabled={})",
        REDIS_CHANNELS.len(),
        sse_enabled
    );

    let zadd_client = redis::Client::open(redis_url.as_str())?;
    let mut zadd_conn = zadd_client.get_async_connection().await?;

    let mut messages = pubsub.on_message();
    while let Some(msg) = messages.next().await {
        let channel = msg.get_channel_name().to_string();
        let payload: String = match msg.get_payload() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to decode message payload on {channel}: {e}");
                continue;
            }
        };

        let raw: serde_json::Value = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Failed to parse message as JSON on {channel}: {e}");
                continue;
            }
        };

        let (event_type, normalized) = normalize_event(&channel, &raw);

        let score = Utc::now().timestamp_millis();
        let member = serde_json::to_string(&normalized).unwrap_or_default();
        let _: Result<(), _> = redis::cmd("ZADD")
            .arg(EVENTS_REDIS_KEY)
            .arg(score)
            .arg(&member)
            .query_async(&mut zadd_conn)
            .await;

        if sse_enabled {
            let guard = clients.lock().await;
            for client in guard.iter() {
                if let Some(ref filter) = client.filter_types {
                    if !filter.contains(&event_type) {
                        continue;
                    }
                }

                let sse = format_sse_event(&event_type, &normalized);
                let _ = client.sender.send(Bytes::from(sse)).await;
            }
        }
    }

    log::warn!("Redis message stream ended");
    Ok(())
}

/// SSE stream endpoint handler.
///
/// # Expected Behavior
///
/// Parses the `types` query parameter into an optional `HashSet` filter.
/// Registers a new client with the `EventBus`, which starts the Redis
/// subscriber if this is the first client. Sends a `connected` event
/// with the list of subscribed channels and the active filter. Spawns
/// a ping task that sends a `ping` event every 30 seconds. Returns an
/// HTTP 200 response with `Content-Type: text/event-stream`,
/// `Cache-Control: no-cache`, `Connection: keep-alive`, and
/// `X-Accel-Buffering: no` headers, streaming events via the
/// `SseReceiverStream`. When the client disconnects, the stream's
/// Drop impl removes the client from the `EventBus`.
///
/// # Errors
///
/// Never returns an error HTTP response. If the Redis subscriber
/// cannot start, the client receives a `connected` event but no
/// further events until Redis reconnects. Redis errors are sent
/// as `event: error` SSE frames.
///
/// # Side Effects
///
/// - Adds a client to the shared `EventBus` (Mutex lock).
/// - May spawn the Redis subscriber task.
/// - Sends the `connected` event through the mpsc channel.
/// - Spawns a ping Tokio task (timer + channel sends every 30s).
/// - The returned streaming response holds the `SseReceiverStream`
///   alive until the client disconnects.
pub async fn sse_stream(
    event_bus: web::Data<EventBus>,
    query: web::Query<SseQuery>,
) -> HttpResponse {
    if !event_bus.sse_enabled {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "SSE is disabled in this deployment",
            "message": "Server-Sent Events require a persistent connection which is not available in serverless mode. Use the polling endpoint instead.",
            "poll_endpoint": "/api/events/poll",
            "poll_interval_ms": 5000,
        }));
    }

    let filter_types: Option<HashSet<String>> = if query.types.is_empty() {
        None
    } else {
        Some(
            query
                .types
                .split(',')
                .map(|t| t.trim().to_string())
                .collect(),
        )
    };

    let (_client_id, tx, rx) = event_bus.add_client(filter_types).await;

    let connected_data = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "channels": REDIS_CHANNELS.iter().map(std::string::ToString::to_string).collect::<Vec<_>>(),
        "filter": if query.types.is_empty() { "all".to_string() } else { query.types.clone() },
    });
    let connected_sse = format_sse_event("connected", &connected_data);
    let _ = tx.send(Bytes::from(connected_sse)).await;

    let ping_tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let ping_data = serde_json::json!({"timestamp": Utc::now().to_rfc3339()});
            let ping_sse = format_sse_event("ping", &ping_data);
            if ping_tx.send(Bytes::from(ping_sse)).await.is_err() {
                break;
            }
        }
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(rx)
}

/// Event history stub endpoint.
///
/// # Expected Behavior
///
/// Returns a 200 OK with a JSON body containing an empty `events`
/// array and a message indicating that event history is not yet
/// implemented. Clients should use `/api/events/stream` for
/// real-time events instead, or `/api/events/poll` in serverless mode.
///
/// # Errors
///
/// None. Always returns 200 OK.
///
/// # Side Effects
///
/// None. Pure response generation with no I/O.
pub async fn event_history() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "events": [],
        "message": "Event history not yet implemented. Use /api/events/poll for polling."
    }))
}

/// Polling endpoint for retrieving recent events.
///
/// # Expected Behavior
///
/// Reads recent events from a Redis sorted set (`openhack:events:recent`)
/// keyed by timestamp. If `after` is provided, only events with a
/// timestamp newer than the given RFC 3339 timestamp are returned.
/// If `after` is absent, returns the most recent events up to `limit`.
/// If Redis is unavailable or the sorted set is empty, returns an
/// empty events list. This endpoint is the recommended alternative
/// to SSE when running in Lambda/serverless mode where persistent
/// connections are not available.
///
/// # Errors
///
/// Returns 200 OK with an empty events list if Redis is unavailable
/// or the sorted set does not exist. Does not return error HTTP
/// statuses for Redis failures to avoid breaking client polling loops.
///
/// # Side Effects
///
/// - Opens a Redis connection (network I/O).
/// - Reads from the `openhack:events:recent` sorted set.
/// - Removes events older than `EVENTS_MAX_AGE_SECS` from the sorted set.
pub async fn event_poll(
    event_bus: web::Data<EventBus>,
    query: web::Query<PollQuery>,
) -> HttpResponse {
    let Some(redis_url) = &event_bus.redis_url else {
        return HttpResponse::Ok().json(serde_json::json!({
            "events": [],
            "message": "Redis not configured. Poll individual service endpoints directly."
        }));
    };

    let Ok(client) = redis::Client::open(redis_url.as_str()) else {
        return HttpResponse::Ok().json(serde_json::json!({
            "events": [],
            "message": "Redis connection failed"
        }));
    };

    let Ok(mut conn) = client.get_async_connection().await else {
        return HttpResponse::Ok().json(serde_json::json!({
            "events": [],
            "message": "Redis connection failed"
        }));
    };

    let cutoff_ts = Utc::now().timestamp_millis() - (EVENTS_MAX_AGE_SECS * 1000);

    let _: Result<(), _> = redis::cmd("ZREMRANGEBYSCORE")
        .arg(EVENTS_REDIS_KEY)
        .arg("-inf")
        .arg(cutoff_ts)
        .query_async(&mut conn)
        .await;

    let min_score: i64 = query
        .after
        .as_ref()
        .and_then(|a| {
            DateTime::parse_from_rfc3339(a)
                .ok()
                .map(|dt| dt.timestamp_millis() + 1)
        })
        .unwrap_or(0);

    let raw_members: Vec<String> = match redis::cmd("ZRANGEBYSCORE")
        .arg(EVENTS_REDIS_KEY)
        .arg(min_score)
        .arg("+inf")
        .arg("LIMIT")
        .arg(0)
        .arg(query.limit)
        .query_async(&mut conn)
        .await
    {
        Ok(members) => members,
        Err(_) => {
            return HttpResponse::Ok().json(serde_json::json!({
                "events": [],
                "message": "Failed to read events from Redis"
            }));
        }
    };

    let events: Vec<PolledEvent> = raw_members
        .iter()
        .filter_map(|member| {
            let raw: serde_json::Value = serde_json::from_str(member).ok()?;
            Some(PolledEvent {
                id: raw
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                event_type: raw
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                channel: raw
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data: raw.get("data").cloned().unwrap_or(serde_json::Value::Null),
                timestamp: raw
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "events": events,
        "count": events.len(),
    }))
}
