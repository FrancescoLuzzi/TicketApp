use crate::auth::mw_auth::CtxResult;
use crate::templates::SignupPage;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Extension;

pub async fn get(Extension(ctx_res): Extension<CtxResult>) -> Response {
    if ctx_res.is_ok() {
        let mut headers = HeaderMap::new();
        headers.append("HX-Redirect", "/home".parse().unwrap());
        (headers, StatusCode::OK).into_response()
    } else {
        SignupPage {}.into_response()
    }
}
