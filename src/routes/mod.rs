use axum::{routing::get, Router};
use sqlx::PgPool;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .with_state(pool)
}

async fn health_handler() -> &'static str {
    "OK"
}
