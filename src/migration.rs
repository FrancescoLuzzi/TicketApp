use sqlx::{migrate, Pool};

pub async fn db_migration<T>(pool: &Pool<T>) -> Result<(), migrate::MigrateError>
where
    T: sqlx::Database,
    <T as sqlx::Database>::Connection: sqlx::migrate::Migrate,
{
    return migrate!("./migrations").run(pool).await;
}
