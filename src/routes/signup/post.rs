use crate::app_state::SharedAppState;
use crate::auth::password::compute_password_hash;
use crate::telemetry::spawn_blocking_with_tracing;
use anyhow::Context;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use std::ops::DerefMut;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct NewUser {
    username: String,
    email: String,
    password: SecretString,
}

#[tracing::instrument(
    name = "new user subscribe",
    skip_all,
    fields(
        sub_email=%new_user.email,
        sub_username=%new_user.username
    ),
)]
pub async fn post(
    State(state): State<SharedAppState>,
    Json(new_user): Json<NewUser>,
) -> Result<StatusCode, StatusCode> {
    let mut transaction = state
        .db_pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_uuid = Uuid::new_v4();
    let hashed_password = hash_password(new_user.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = sqlx::query!(
        "INSERT INTO tbl_user (id,username, email,password) VALUES ($1, $2, $3, $4)",
        new_uuid,
        new_user.username,
        new_user.email,
        hashed_password.expose_secret()
    )
    .execute(transaction.deref_mut())
    .await;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

async fn hash_password(password: SecretString) -> Result<SecretString, anyhow::Error> {
    spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password")
}
