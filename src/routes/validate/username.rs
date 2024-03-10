use crate::app_state::SharedAppState;
use crate::templates::validation::username::UsernameValidation;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;

#[derive(serde::Deserialize)]
pub struct UserReq {
    username: String,
}

pub async fn post(
    State(state): State<SharedAppState>,
    Form(user_req): Form<UserReq>,
) -> Result<UsernameValidation, StatusCode> {
    let result = sqlx::query!(
        "SELECT id FROM tbl_user WHERE username = $1",
        user_req.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed new user subscription: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(UsernameValidation {
        is_valid: !result.is_some(),
    })
}
