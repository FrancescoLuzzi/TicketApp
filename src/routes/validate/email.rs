use crate::app_state::SharedAppState;
use crate::templates::validation::form::FormValidation;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct UserReq {
    #[validate(email)]
    email: String,
}

pub async fn post(
    State(state): State<SharedAppState>,
    Form(user_req): Form<UserReq>,
) -> Result<FormValidation<'static>, StatusCode> {
    let mut resp = FormValidation {
        target: "email-error",
        valid_message: "valid email",
        invalid_message: "",
        is_valid: false,
    };
    if user_req.validate().is_err() {
        resp.invalid_message = "invalid email";
        return Ok(resp);
    }
    let result = sqlx::query!("SELECT id FROM tbl_user WHERE email = $1", user_req.email)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed new user subscription: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    resp.invalid_message = "already used";
    resp.is_valid = result.is_none();
    Ok(resp)
}
