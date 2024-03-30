use super::mw_auth::AUTH_COOKIE;
use crate::{app_state::RedisConnection, ctx::Ctx};
use redis::AsyncCommands;
use tower_cookies::Cookies;

pub async fn logout(
    ctx: Ctx,
    cookies: Cookies,
    mut redis_connection: RedisConnection<'_>,
) -> anyhow::Result<()> {
    cookies.remove(AUTH_COOKIE.into());
    redis_connection.del(ctx.session_id()).await?;
    Ok(())
}
