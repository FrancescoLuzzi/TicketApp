use std::{net::SocketAddr, sync::Arc};

use askama_axum::IntoResponse;
use axum::{
    self, middleware,
    response::Response,
    routing::{get, post},
    Router,
};
use bb8_redis::bb8;
use bb8_redis::RedisConnectionManager;
use sqlx::postgres::PgPoolOptions;
use ticket_app::{
    app_state::{AppState, SharedAppState},
    auth::mw_auth,
    configuration::load_settings,
    routes::{health_check, home, index, login, logout, signup, ticket, validate},
    telemetry::{get_subscriber, init_subscriber},
    migration::db_migration
};
use tower_cookies::CookieManagerLayer;
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
    db_migration(&db_pool).await.expect("migration script failed");
    let app_state: SharedAppState = Arc::new(AppState {
        redis_pool,
        db_pool,
        hmac_secret: settings.application.hmac_secret,
        base_url: settings.application.base_url,
    });
    let serve_dir = ServeDir::new("dist");

    let app = Router::new()
        .route("/ticket", get(ticket::get))
        .route("/ticket", post(ticket::post))
        .route_layer(middleware::from_fn(mw_auth::mw_ctx_require))
        .route("/home", get(home::get))
        .route("/logout", post(logout::post))
        .route("/login", get(login::get))
        .route("/login", post(login::post))
        .route("/signup", post(signup::post))
        .route("/signup", get(signup::get))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_auth::mw_ctx_resolver,
        ))
        .route("/", get(index))
        .route("/favicon.ico", get(favicon))
        .route("/health_check", get(health_check))
        .route("/validation/username", post(validate::username::post))
        .route("/validation/email", post(validate::email::post))
        .layer(CookieManagerLayer::new())
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
