use std::{net::SocketAddr, sync::Arc};

use axum::{
    self,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use ticket_app::routes::health_check;
use ticket_app::telemetry::{get_subscriber, init_subscriber};
use ticket_app::{
    app_state::AppState,
    routes::{login, signup},
};
use ticket_app::{configuration::load_settings, routes::index};
use tracing_log::log::Level;

#[tokio::main]
async fn main() {
    let settings = load_settings();
    let settings = settings.unwrap();
    let telemetry_subscriber =
        get_subscriber("ticket_app".to_string(), Level::Info, settings.logging);
    init_subscriber(telemetry_subscriber);
    let db_pool = PgPoolOptions::new().connect_lazy_with(settings.database.with_db());
    let app_state = Arc::new(AppState {
        db_pool,
        hmac_secret: settings.application.hmac_secret,
        base_url: settings.application.base_url,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/health_check", get(health_check))
        .route("/signup", post(signup::post))
        .route("/login", get(login::get))
        .with_state(app_state);
    let addr = SocketAddr::new(settings.application.host, settings.application.port);
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
