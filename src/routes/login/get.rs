use crate::app_state::SharedAppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use secrecy::SecretString;
use std::ops::DerefMut;
use uuid::Uuid;

#[derive(sqlx::FromRow, askama::Template)]
#[template(path = "user_read.html")]
struct User {
    id: Uuid,
    username: String,
    password: SecretString,
    email: String,
}

#[derive(askama::Template)]
#[template(
    source = r#"
<div class="users_container>">
    {% for item in users %}
    {{ item }}
    {% endfor %}
</div>
"#,
    ext = "html",
    escape = "none"
)]
struct Users<'a> {
    users: &'a Vec<User>,
}

#[tracing::instrument(skip_all)]
pub async fn get(State(state): State<SharedAppState>) -> Result<impl IntoResponse, StatusCode> {
    let mut transaction = state
        .clone()
        .db_pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let users = sqlx::query_as!(User, "SELECT * FROM tbl_user")
        .fetch_all(transaction.deref_mut())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let users_out = Users { users: &users };
    Ok((StatusCode::OK, users_out.into_response()))
}
