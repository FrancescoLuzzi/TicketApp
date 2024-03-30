use bb8_redis::{
    bb8::{Pool, PooledConnection},
    RedisConnectionManager,
};
use secrecy::SecretString;
use sqlx::PgPool;
use std::sync::Arc;

pub type SharedAppState = Arc<AppState>;
pub type RedisPool = Pool<RedisConnectionManager>;
pub type RedisConnection<'a> = PooledConnection<'a, RedisConnectionManager>;

pub struct AppState {
    pub db_pool: PgPool,
    pub redis_pool: RedisPool,
    pub hmac_secret: SecretString,
    pub base_url: String,
}
