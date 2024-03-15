use bb8_redis::{bb8::Pool, RedisConnectionManager};
use secrecy::SecretString;
use sqlx::PgPool;
use std::sync::Arc;

pub type SharedAppState = Arc<AppState>;

pub struct AppState {
    pub db_pool: PgPool,
    pub redis_pool: Pool<RedisConnectionManager>,
    pub hmac_secret: SecretString,
    pub base_url: String,
}
