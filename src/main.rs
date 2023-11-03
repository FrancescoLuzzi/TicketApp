use std::net::SocketAddr;

use axum::{
    self,
    routing::{get, post},
    Router,
};
use ticket_app::configuration::load_settings;
use ticket_app::routes::health_check;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() {
    let settings = load_settings().unwrap();

    let app = Router::new()
        .route("/health_check", get(health_check))
        .layer(CookieManagerLayer::new());
    let addr = SocketAddr::new(settings.application.host, settings.application.port);
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
