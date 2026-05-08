use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

pub async fn create_pool(database_url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    MIGRATOR.run(&pool).await?;
    info!("database migrations applied");

    Ok(pool)
}
