use std::net::SocketAddr;

use axum::{
    self,
    routing::{get, post},
    Router,
};
use ticket_app::configuration::load_settings;
use ticket_app::routes::health_check;
use ticket_app::telemetry::{get_subscriber, init_subscriber};
use tower_cookies::CookieManagerLayer;
use tracing_log::log::Level;

#[tokio::main]
async fn main() {
    let settings = load_settings().unwrap();
    let telemetry_subscriber =
        get_subscriber("ticket_app".to_string(), Level::Info, std::io::stdout);
    init_subscriber(telemetry_subscriber);

    let app = Router::new()
        .route("/health_check", get(health_check))
        .layer(CookieManagerLayer::new());
    let addr = SocketAddr::new(settings.application.host, settings.application.port);
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
