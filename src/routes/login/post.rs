use crate::{
    app_state::SharedAppState,
    auth::{
        mw_auth::{CtxResult, AUTH_COOKIE},
        password::{validate_credentials, Credentials},
        session_key::generate_session_key,
    },
};
use askama_axum::IntoResponse;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Redirect, Response},
    Form,
};
use bb8_redis::redis::{AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};
use tower_cookies::{Cookie, Cookies};

pub async fn post(
    State(state): State<SharedAppState>,
    ctx_res: Extension<CtxResult>,
    cookies: Cookies,
    Form(credentials): Form<Credentials>,
) -> Response {
    if ctx_res.is_ok() {
        return Redirect::to("/home").into_response();
    }
    match validate_credentials(credentials, &state.db_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            let mut conn = state.redis_pool.get().await.unwrap();
            let opts = SetOptions::default()
                .conditional_set(ExistenceCheck::NX)
                .with_expiration(SetExpiry::EX(60));
            // add session in redis
            let session_key = generate_session_key();
            let _: () = conn.set_options(&session_key, user_id, opts).await.unwrap();
            cookies.add(Cookie::new(AUTH_COOKIE, session_key.as_ref().to_string()));
            Redirect::to("/home").into_response()
        }
        Err(_) => (StatusCode::BAD_REQUEST, Redirect::to("/login")).into_response(),
    }
}
