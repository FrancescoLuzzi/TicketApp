use crate::app_state::SharedAppState;
use crate::auth::password::{hash_password, is_password_strong};
use crate::templates::validation::password::PasswordValidation;
use askama_axum::{IntoResponse, Response};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Form,
};
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

#[derive(Debug, sqlx::FromRow)]
pub struct NewUserUuid {
    id: uuid::Uuid,
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
        return Ok((PasswordValidation {}).into_response());
    }
    let mut transaction = state
        .db_pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let hashed_password = hash_password(new_user.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_user_uuid = sqlx::query_as!(
        NewUserUuid,
        r#"INSERT INTO tbl_user (username, email, password) VALUES ($1, $2, $3) RETURNING id"#,
        new_user.username,
        new_user.email,
        hashed_password.expose_secret()
    )
    .fetch_one(transaction.deref_mut())
    .await
    .map_err(|e| {
        tracing::error!("Failed new user subscription: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "INSERT INTO tbl_accounting (name, user_id, description) VALUES ('default', $1, 'your first tbl_accounting')",
        new_user_uuid.id,
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
    Ok((headers, StatusCode::OK).into_response())
}
