mod config;
mod db;
mod errors;
mod routes;
use config::Config;
mod jwt;
mod middleware;
mod models;

#[tokio::main]
async fn main() {
    // Load config from .env
    let config = Config::load();

    // Setup logging
    tracing_subscriber::fmt::init();

    // Connect to database
    let pool = db::connect(&config).await;
    tracing::info!("Connected to database successfully");

    // Build app
    let app = routes::create_router(pool);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
