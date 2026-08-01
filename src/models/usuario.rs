use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Usuario {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PerfilResponse {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub fecha_registro: DateTime<Utc>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UsuarioPublicoResponse {
    pub id_usuario: i32,
    pub username: String,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsuarioListItem {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub fecha_registro: DateTime<Utc>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Username o email
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
    /// URL ya subida a Cloudinary (opcional). Si no viene, se usa el avatar default.
    pub url_avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsuarioListQuery {
    pub q: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
