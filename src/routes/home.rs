use askama_axum::{IntoResponse, Response};
use axum::{response::Redirect, Extension};

use crate::{auth::mw_auth::CtxResult, templates::Home};

pub async fn get(Extension(ctx_res): Extension<CtxResult>) -> Response {
    match ctx_res {
        Ok(ctx) => Home {
            user: ctx.user_id().to_string().into(),
        }
        .into_response(),
        Err(_) => Redirect::to("/login").into_response(),
    }
}
