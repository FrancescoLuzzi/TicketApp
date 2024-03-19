use askama_axum::{IntoResponse, Response};
use axum::{response::Redirect, Extension};

use crate::{auth::mw_auth::CtxResult, templates::Login};

pub async fn get(ctx_res: Extension<CtxResult>) -> Response {
    if ctx_res.is_ok() {
        Redirect::to("/home").into_response()
    } else {
        Login {}.into_response()
    }
}
