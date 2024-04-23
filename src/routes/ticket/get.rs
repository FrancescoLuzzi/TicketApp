use crate::model::direction::TicketDirection;
use crate::{app_state::SharedAppState, auth::mw_auth::CtxResult};
use axum::{extract::State, http::StatusCode, response::Extension, Json};
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct Ticket {
    direction: TicketDirection,
    #[serde(with = "rust_decimal::serde::float")]
    amount: sqlx::types::Decimal,
    r#type: String,
    type_id: Uuid,
    parent_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    description: String,
}

pub async fn get(
    State(state): State<SharedAppState>,
    Extension(ctx_res): Extension<CtxResult>,
) -> Result<Json<Vec<Ticket>>, StatusCode> {
    if ctx_res.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let tickets: Vec<_> = sqlx::query_as!(
        Ticket,
        r#"SELECT 
            amt.direction as "direction!: TicketDirection",
            amt.amount,
            amt.description,
            amt.created_at,
            tt.name as type,
            tt.id as "type_id!: Uuid",
            tt.parent_id as "parent_id!: Option<Uuid>"
        FROM accounting_movement_tbl amt
          INNER JOIN tbl_type tt ON tt.id = amt.type_id
          INNER JOIN tbl_accounting ta ON ta.id = amt.accounting_id
        WHERE
            ta.user_id = $1"#,
        ctx_res.unwrap().user_id()
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed new user subscription: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .collect();
    Ok(tickets.into())
}
