use crate::app_state::SharedAppState;
use crate::auth::password::{hash_password, is_password_strong};
use crate::templates::validation::password::PasswordValidation;
use askama_axum::into_response;
use askama_axum::{IntoResponse, Response};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Form;
use secrecy::{ExposeSecret, SecretString};
use std::ops::DerefMut as _;
use validator::Validate;

#[derive(Debug, Validate, serde::Deserialize)]
pub struct NewUser {
    username: String,
    #[validate(email)]
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
        return Ok((StatusCode::BAD_REQUEST, "password confirm error").into_response());
    }
    if new_user.validate().is_err() {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
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
    let mut headers = HeaderMap::new();
    headers.append("HX-Redirect", "/login".parse().unwrap());
    Ok((headers, StatusCode::NO_CONTENT).into_response())
}
