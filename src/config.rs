// config.rs — App configuration
//
// This holds all the values our app needs to run.
// Instead of scattering these values everywhere,
// we load them once here and pass them around.

// Clone lets us pass Config to multiple places (like routes and middleware)
#[derive(Clone)]
pub struct Config {
    pub database_url: String, // connection string for Postgres
    pub jwt_secret: String,   // secret key used to sign and verify JWT tokens
}

impl Config {
    pub fn load() -> Self {
        Self {
            // This is the URL to connect to our Docker Postgres
            // Format: postgres://username:password@host:port/database
            database_url: "postgresql://postgres:pass123@127.0.0.1:5432/iam_db?sslmode=disable"
                .to_string(),

            // This is used to sign JWT tokens
            // In production this should be a long random string stored securely
            jwt_secret: "super_secret_key_change_in_production".to_string(),
        }
    }
}
