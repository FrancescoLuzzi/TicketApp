use crate::model::direction::TicketDirection;
use crate::{app_state::SharedAppState, auth::mw_auth::CtxResult};
use axum::{extract::State, http::StatusCode, response::Extension, Json};
use sqlx::types::chrono;

#[derive(Debug, serde::Deserialize)]
pub struct Ticket {
    direction: TicketDirection,
    #[serde(with = "rust_decimal::serde::float")]
    amount: sqlx::types::Decimal,
    type_id: uuid::Uuid,
    description: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    //accounting_id: uuid::Uuid
}

pub async fn post(
    State(state): State<SharedAppState>,
    Extension(ctx_res): Extension<CtxResult>,
    Json(ticket): Json<Ticket>,
) -> Result<StatusCode, StatusCode> {
    if ctx_res.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    sqlx::query!(
        r#"INSERT INTO accounting_movement_tbl(accounting_id,type_id,direction,amount,description,created_at)
        SELECT id,$1,$2,$3,$4,$5
        FROM tbl_accounting
        WHERE
          name = 'default' AND user_id=$6"#,
        ticket.type_id,
        ticket.direction as TicketDirection,
        ticket.amount,
        ticket.description,
        ticket.created_at.unwrap_or(chrono::Utc::now()),
        ctx_res.unwrap().user_id(),
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed new user subscription: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::OK)
}
