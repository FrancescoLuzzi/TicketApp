use sqlx::{migrate, Pool};

#[tracing::instrument]
pub async fn db_migration<T>(pool: &Pool<T>) -> Result<(), migrate::MigrateError>
where
    T: sqlx::Database,
    <T as sqlx::Database>::Connection: sqlx::migrate::Migrate,
{
    migrate!().run(pool).await
}
