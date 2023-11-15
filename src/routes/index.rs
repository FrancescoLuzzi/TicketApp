use axum::response::IntoResponse;

pub async fn index() -> impl IntoResponse {
    "Hello world!"
}
