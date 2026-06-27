use crate::config::Config;
use sqlx::postgres::PgPoolOptions;

pub async fn connect(config: &Config) -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres")
}
