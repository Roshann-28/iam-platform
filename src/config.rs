pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            database_url: "postgresql://postgres:pass123@127.0.0.1:5432/iam_db?sslmode=disable"
                .to_string(),
            jwt_secret: "super_secret_key_change_in_production".to_string(),
        }
    }
}
