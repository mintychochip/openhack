use actix_web::web;
use redis::Client;

pub type RedisConn = web::Data<Option<redis::aio::MultiplexedConnection>>;

pub async fn create_connection(redis_url: &str) -> RedisConn {
    if redis_url.is_empty() {
        log::warn!("REDIS_URL not set. Running without Redis (caching and events disabled).");
        return web::Data::new(None);
    }

    match Client::open(redis_url) {
        Ok(client) => match client.get_multiplexed_async_connection().await {
            Ok(conn) => {
                log::info!("Redis connection established");
                web::Data::new(Some(conn))
            }
            Err(e) => {
                log::warn!("Redis connection failed: {e}. Running without Redis.");
                web::Data::new(None)
            }
        },
        Err(e) => {
            log::warn!("Redis client creation failed: {e}. Running without Redis.");
            web::Data::new(None)
        }
    }
}
