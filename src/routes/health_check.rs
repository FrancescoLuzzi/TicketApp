use axum::http::StatusCode;
use axum::response::IntoResponse;

#[tracing::instrument]
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
