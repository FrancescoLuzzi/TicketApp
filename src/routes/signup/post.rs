use crate::auth::password::compute_password_hash;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::templates::validation::password::PasswordValidation;
use crate::{app_state::SharedAppState, auth::password::is_password_strong};
use anyhow::Context;
use askama_axum::into_response;
use askama_axum::{IntoResponse, Response};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use secrecy::{ExposeSecret, SecretString};
use std::ops::DerefMut as _;

#[derive(serde::Deserialize)]
pub struct NewUser {
    username: String,
    email: String,
    password: SecretString,
    #[serde(rename = "password-confirm")]
    password_confirm: SecretString,
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
    Form(new_user): Form<NewUser>,
) -> Result<Response, StatusCode> {
    if new_user.password.expose_secret() != new_user.password_confirm.expose_secret() {
        return Ok((StatusCode::OK, "password confirm error").into_response());
    }
    if !is_password_strong(&new_user.password) {
        return Ok(into_response(&PasswordValidation {}));
    }
    let mut transaction = state
        .db_pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let hashed_password = hash_password(new_user.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query!(
        "INSERT INTO tbl_user (username, email, password) VALUES ($1, $2, $3)",
        new_user.username,
        new_user.email,
        hashed_password.expose_secret()
    )
    .execute(transaction.deref_mut())
    .await
    .map_err(|e| {
        tracing::error!("Failed new user subscription: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    transaction.commit().await.map_err(|e| {
        tracing::error!("Failed committing transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::OK.into_response())
}

async fn hash_password(password: SecretString) -> Result<SecretString, anyhow::Error> {
    spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password")
}
