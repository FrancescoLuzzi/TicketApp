use axum::{
    http::{HeaderMap, StatusCode},
    response::{Extension, IntoResponse, Response},
};

use crate::{auth::mw_auth::CtxResult, templates::LoginPage};

pub async fn get(Extension(ctx_res): Extension<CtxResult>) -> Response {
    if ctx_res.is_ok() {
        let mut headers = HeaderMap::new();
        headers.append("HX-Redirect", "/home".parse().unwrap());
        (headers, StatusCode::OK).into_response()
    } else {
        LoginPage {}.into_response()
    }
}
