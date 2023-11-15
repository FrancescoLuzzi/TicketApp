use secrecy::SecretString;
use sqlx::PgPool;
use std::sync::Arc;

pub type SharedAppState = Arc<AppState>;

pub struct AppState {
    pub db_pool: PgPool,
    pub hmac_secret: SecretString,
    pub base_url: String,
}
