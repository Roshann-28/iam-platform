// db.rs — Database connection
//
// Instead of connecting to the database on every request (slow),
// we create a "pool" of connections once at startup and reuse them.
// Think of it like a pool of workers — requests grab a free worker,
// use it, then return it to the pool.

use sqlx::postgres::PgPoolOptions; // lets us configure the connection pool
use sqlx::PgPool; // the pool type we'll use everywhere

use crate::config::Config; // bring our Config in

// connect() creates the pool and returns it
// It's async because connecting to a database takes time
pub async fn connect(config: &Config) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5) // max 5 connections open at once
        .connect(&config.database_url) // connect using our URL
        .await
        .expect("Failed to connect to database") // crash if connection fails
}
