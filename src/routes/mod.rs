// routes/mod.rs — Router
//
// This file wires everything together.
// It defines which URL paths go to which handler functions.
//
// Our routes:
// GET  /health         — public, check if server is running
// POST /auth/register  — public, create a new account
// POST /auth/login     — public, login to an existing account
// GET  /me             — protected, get the logged-in user's info
// POST /orgs           — protected, create an organization
// GET  /orgs           — protected, list user's organizations
// GET  /orgs/:id       — protected, get a specific organization
// DELETE /orgs/:id     — protected, delete an organization (owner only)

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post},
    Extension, Json, Router,
};

use crate::middleware::AuthUser;

mod auth;
mod org;

// AppState — shared data available to every handler
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: crate::config::Config,
}

// create_router — builds and returns the complete router
pub fn create_router(pool: sqlx::PgPool, config: crate::config::Config) -> Router {
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // All protected routes go here
    // auth_middleware runs before every route in this group
    let protected_routes = Router::new()
        .route("/me", get(me_handler))
        .route("/orgs", post(org::create_org).get(org::list_orgs))
        .route("/orgs/{id}", get(org::get_org).delete(org::delete_org))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    Router::new()
        .route("/health", get(health_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .merge(protected_routes)
        .with_state(state)
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
