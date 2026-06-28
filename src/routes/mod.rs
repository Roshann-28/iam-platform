// routes/mod.rs — Router
//
// This file wires everything together.
// It defines which URL paths go to which handler functions.

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Json, Router,
};

use crate::middleware::AuthUser;

mod auth;

// AppState — shared data available to every handler
// We wrap pool and config together so we can pass them as one unit
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,            // database connection pool
    pub config: crate::config::Config, // app config (JWT secret etc)
}

// create_router — builds and returns the complete router
pub fn create_router(pool: sqlx::PgPool, config: crate::config::Config) -> Router {
    // Bundle pool and config into one state object
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Protected routes — require a valid JWT token
    let protected_routes = Router::new().route("/me", get(me_handler)).route_layer(
        axum_middleware::from_fn_with_state(state.clone(), crate::middleware::auth_middleware),
    );

    // Build the final router
    Router::new()
        .route("/health", get(health_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .merge(protected_routes)
        .with_state(state) // attach AppState to all routes
}

// health_handler — GET /health
async fn health_handler() -> &'static str {
    "OK"
}

// me_handler — GET /me (protected)
async fn me_handler(Extension(auth_user): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": auth_user.user_id,
        "email": auth_user.email
    }))
}
