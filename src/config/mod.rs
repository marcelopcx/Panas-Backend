use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub host: String,
    pub port: u16,
    pub cloudinary: CloudinaryConfig,
    pub default_avatar_url: String,
}

#[derive(Clone)]
pub struct CloudinaryConfig {
    pub cloud_name: String,
    pub upload_preset: String,
    pub folder: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let cloudinary = CloudinaryConfig {
            cloud_name: env::var("CLOUDINARY_CLOUD_NAME")
                .unwrap_or_else(|_| "mpc-uru".to_string()),
            upload_preset: env::var("CLOUDINARY_UPLOAD_PRESET")
                .unwrap_or_else(|_| "n3n6sbhv".to_string()),
            folder: env::var("CLOUDINARY_FOLDER").unwrap_or_else(|_| "panas".to_string()),
        };

        let default_avatar_url = env::var("DEFAULT_AVATAR_URL").unwrap_or_else(|_| {
            format!(
                "https://res.cloudinary.com/{}/image/upload/{}/avatars/default.jpg",
                cloudinary.cloud_name, cloudinary.folder
            )
        });

        Self {
            database_url: env::var("DATABASE_URL").expect("falta DATABASE_URL en .env"),
            jwt_secret: env::var("JWT_SECRET").expect("falta JWT_SECRET en .env"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse::<i64>()
                .expect("JWT_EXPIRATION_HOURS debe ser un número (ej: 24)"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            // Render inyecta PORT; localmente default 8080.
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse::<u16>()
                .expect("PORT debe ser un número (ej: 8080)"),
            cloudinary,
            default_avatar_url,
        }
    }
}
