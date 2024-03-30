use crate::{
    app_state::SharedAppState,
    auth::{logout, mw_auth::CtxResult},
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Extension, IntoResponse, Response},
};
use tower_cookies::Cookies;

pub async fn post(
    State(state): State<SharedAppState>,
    Extension(ctx_res): Extension<CtxResult>,
    cookies: Cookies,
) -> Result<Response, StatusCode> {
    if ctx_res.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let conn = state
        .redis_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match logout::logout(ctx_res.unwrap(), cookies, conn).await {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.append("HX-Redirect", "/home".parse().unwrap());
            Ok((headers, StatusCode::OK).into_response())
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}
