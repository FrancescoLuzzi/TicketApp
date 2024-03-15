use crate::app_state::SharedAppState;
use crate::auth::password::{validate_credentials, Credentials};
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Redirect, Response};
use axum::Form;
use bb8_redis::redis::{AsyncCommands, SetExpiry, SetOptions};

pub async fn post(
    State(state): State<SharedAppState>,
    Form(credentials): Form<Credentials>,
) -> Response {
    match validate_credentials(credentials, &state.db_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            let mut conn = state.redis_pool.get().await.unwrap();
            let opts = SetOptions::default().with_expiration(SetExpiry::EX(60));
            // add session in redis
            let _: () = conn
                .set_options(user_id.to_string(), 10, opts)
                .await
                .unwrap();
            Redirect::to("/home").into_response()
        }
        Err(_) => (StatusCode::BAD_REQUEST, Redirect::to("/login")).into_response(),
    }
}
