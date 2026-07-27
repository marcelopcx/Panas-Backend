use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").expect("falta DATABASE_URL en .env"),
            jwt_secret: env::var("JWT_SECRET").expect("falta JWT_SECRET en .env"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .expect("falta JWT_EXPIRATION_HOURS en .env")
                .parse::<i64>()
                .expect("JWT_EXPIRATION_HOURS debe ser un número (ej: 24)"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .expect("falta PORT en .env")
                .parse::<u16>()
                .expect("PORT debe ser un número (ej: 8080)"),
        }
    }
}
