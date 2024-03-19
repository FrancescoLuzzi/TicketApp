use crate::{app_state::SharedAppState, ctx::Ctx};
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use bb8_redis::redis::AsyncCommands;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use super::session_key::SessionKey;

pub type CtxResult = Result<Ctx, CtxExtError>;
pub const AUTH_COOKIE: &str = "x-session";

pub async fn mw_ctx_require(
    ctx: CtxResult,
    req: Request<Body>,
    next: Next,
) -> Result<Response, CtxExtError> {
    dbg!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    State(state): State<SharedAppState>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    dbg!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_ext_result = ctx_resolve(state, &cookies).await;

    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_COOKIE))
    }

    // Store the ctx_ext_result in the request extension
    // (for Ctx extractor).
    req.extensions_mut().insert(ctx_ext_result);

    next.run(req).await
}

async fn ctx_resolve(state: SharedAppState, cookies: &Cookies) -> CtxResult {
    let mut conn = state
        .redis_pool
        .get()
        .await
        .map_err(|_| CtxExtError::SessionAccessError)?;
    let session_id: SessionKey = cookies
        .get(AUTH_COOKIE)
        .ok_or(CtxExtError::TokenNotInCookie)?
        .value()
        .try_into()
        .map_err(|_| CtxExtError::TokenMalformed)?;
    let user_id: Uuid = conn
        .get_ex(session_id, redis::Expiry::EX(10))
        .await
        .map_err(|_| CtxExtError::SessionNotFound)?;
    Ctx::new(user_id).map_err(|_| CtxExtError::CtxCreateFail(user_id.to_string()))
}

#[derive(Clone, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenMalformed,
    SessionNotFound,
    SessionAccessError,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
