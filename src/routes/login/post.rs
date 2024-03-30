use tower_cookies::cookie::time::Duration;

use crate::{
    app_state::SharedAppState,
    auth::{
        mw_auth::{CtxResult, AUTH_COOKIE},
        password::{validate_credentials, Credentials},
        session_key::generate_session_key,
    },
};
use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
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
        return StatusCode::OK.into_response();
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
            let mut auth_cookie = Cookie::new(AUTH_COOKIE, session_key.as_ref().to_string());
            auth_cookie.set_max_age(Duration::seconds(10));
            auth_cookie.set_http_only(true);
            cookies.add(auth_cookie);
            let mut headers = HeaderMap::new();
            headers.append("HX-Redirect", "/home".parse().unwrap());
            (headers, StatusCode::OK).into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
