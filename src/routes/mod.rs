use axum::{middleware as axum_middleware, routing::get, routing::post, Router};
use sqlx::PgPool;

mod auth;

pub fn create_router(pool: PgPool) -> Router {
    let protected = Router::new().route("/me", get(me_handler)).route_layer(
        axum_middleware::from_fn_with_state(pool.clone(), crate::middleware::auth_middleware),
    );

    Router::new()
        .route("/health", get(health_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .merge(protected)
        .with_state(pool)
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn me_handler(
    axum::Extension(auth_user): axum::Extension<crate::middleware::AuthUser>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "user_id": auth_user.user_id,
        "email": auth_user.email
    }))
}
