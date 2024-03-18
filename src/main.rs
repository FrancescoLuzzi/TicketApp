use std::{net::SocketAddr, sync::Arc};

use askama_axum::IntoResponse;
use axum::{
    self,
    response::Response,
    routing::{get, post},
    Router,
};
use bb8_redis::bb8;
use bb8_redis::RedisConnectionManager;
use sqlx::postgres::PgPoolOptions;
use ticket_app::routes::health_check;
use ticket_app::telemetry::{get_subscriber, init_subscriber};
use ticket_app::{
    app_state::AppState,
    routes::validate,
    routes::{login, signup},
};
use ticket_app::{configuration::load_settings, routes::index};
use tower_http::services::ServeDir;
use tracing_log::log::Level;

async fn favicon() -> Response {
    include_bytes!("../favicon.ico").into_response()
}

#[tokio::main]
async fn main() {
    let settings = load_settings();
    let settings = settings.unwrap();
    let telemetry_subscriber =
        get_subscriber("ticket_app".to_string(), Level::Info, settings.logging);
    init_subscriber(telemetry_subscriber);
    let redis_manager = RedisConnectionManager::new(settings.redis.with_db()).unwrap();
    let redis_pool = bb8::Pool::builder()
        .min_idle(3)
        .build(redis_manager)
        .await
        .unwrap();
    let db_pool = PgPoolOptions::new().connect_lazy_with(settings.database.with_db());
    let app_state = Arc::new(AppState {
        redis_pool,
        db_pool,
        hmac_secret: settings.application.hmac_secret,
        base_url: settings.application.base_url,
    });
    let serve_dir = ServeDir::new("dist");

    let app = Router::new()
        .route("/login", get(login::get))
        .route("/login", post(login::post))
        .route("/", get(index))
        .route("/favicon.ico", get(favicon))
        .route("/health_check", get(health_check))
        .route("/signup", post(signup::post))
        .route("/signup", get(signup::get))
        .route("/validation/username", post(validate::username::post))
        .route("/validation/email", post(validate::email::post))
        .nest_service("/dist", serve_dir)
        .with_state(app_state);

    let addr = SocketAddr::new(settings.application.host, settings.application.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap()
}
