use axum::{routing::get, routing::post, Router};
use sqlx::PgPool;

mod auth;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .with_state(pool)
}

async fn health_handler() -> &'static str {
    "OK"
}
