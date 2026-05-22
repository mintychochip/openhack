#![allow(clippy::float_arithmetic, clippy::arithmetic_side_effects)]

use std::sync::{LazyLock, Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{LocalBoxFuture, ok, Ready};
use futures::FutureExt;
use prometheus::{register_int_counter_vec, register_histogram_vec, register_gauge, IntCounterVec, HistogramVec, Gauge, Encoder, TextEncoder, IntCounter};

const HISTOGRAM_BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0];

static HTTP_REQUESTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        "http_requests_total",
        "Total HTTP requests processed",
        &["service", "status"]
    ).expect("http_requests_total can be created")
});

static HTTP_REQUESTS_FAILED: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        "http_requests_failed",
        "Total HTTP requests that failed",
        &["service", "status"]
    ).expect("http_requests_failed can be created")
});

static REQUEST_DURATION: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["service"],
        HISTOGRAM_BUCKETS.iter().copied().collect()
    ).expect("http_request_duration_seconds can be created")
});

static PROCESS_UPTIME: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "process_uptime_seconds",
        "Service uptime in seconds"
    ).expect("process_uptime_seconds can be created")
});

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

static BUSINESS_METRICS: LazyLock<Arc<Mutex<HashMap<String, IntCounter>>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

pub fn register_business_counter(name: &str, help: &str) {
    let counter = IntCounter::with_opts(
        prometheus::Opts::new(name, help)
    ).expect("business counter can be created");
    prometheus::register(Box::new(counter.clone())).expect("business counter can be registered");
    let mut metrics = BUSINESS_METRICS.lock().unwrap();
    metrics.insert(name.to_string(), counter);
}

pub fn inc_business_counter(name: &str) {
    let metrics = BUSINESS_METRICS.lock().unwrap();
    if let Some(counter) = metrics.get(name) {
        counter.inc();
    }
}

pub fn record_request(service: &str, status: u16, duration_secs: f64) {
    let status_str = status.to_string();
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[service, &status_str])
        .inc();
    REQUEST_DURATION
        .with_label_values(&[service])
        .observe(duration_secs);
    if status >= 400 {
        HTTP_REQUESTS_FAILED
            .with_label_values(&[service, &status_str])
            .inc();
    }
}

pub fn update_uptime() {
    PROCESS_UPTIME.set(START_TIME.elapsed().as_secs_f64());
}

#[must_use]
pub fn render_metrics() -> String {
    update_uptime();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut output = Vec::new();
    encoder.encode(&metric_families, &mut output).expect("encoding failed");
    String::from_utf8(output).expect("invalid utf8")
}

pub struct MetricsMiddleware {
    service: String,
}

impl MetricsMiddleware {
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddlewareService {
            service,
            service_name: self.service.clone(),
        })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
    service_name: String,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let service_name = self.service_name.clone();
        let fut = self.service.call(req);

        async move {
            let res = fut.await?;
            let elapsed = start.elapsed().as_secs_f64();
            let status = res.status().as_u16();
            record_request(&service_name, status, elapsed);
            Ok(res)
        }
        .boxed_local()
    }
}
