#[derive(Clone)]
pub struct AppConfig {
    pub server_address: String,
    pub server_port: u16,
    pub oldest_level: i32,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server_address: std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "localhost".to_owned()),
            server_port: std::env::var("SERVER_PORT").expect("SERVER_PORT must be a valid u16").parse().unwrap(),
            oldest_level: std::env::var("OLDEST_LEVEL").expect("OLDEST_LEVEL must be a valid i32").parse().unwrap(),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}