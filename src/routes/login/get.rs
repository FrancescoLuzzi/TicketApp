use crate::app_state::SharedAppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::ops::DerefMut;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct User {
    id: Uuid,
    username: String,
    password: String,
    email: String,
}

pub async fn get(State(state): State<SharedAppState>) -> Result<impl IntoResponse, StatusCode> {
    let mut transaction = state.clone()
        .db_pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let users = sqlx::query_as!(User, "SELECT * FROM tbl_user")
        .fetch_all(transaction.deref_mut())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    transaction.commit().await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::OK, Json(users)))
}
