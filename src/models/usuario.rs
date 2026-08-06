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
    /// Nombre completo para la UI (equivalente a `fullName` del front).
    pub name: String,
    pub fecha_registro: DateTime<Utc>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
    /// `publico` | `privado` | `solo_amigos` (UI: Público / Privado / Solo amigos)
    pub privacidad: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UsuarioPublicoResponse {
    pub id_usuario: i32,
    pub username: String,
    pub url_avatar: Option<String>,
    pub name: String,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
    pub privacidad: String,
}

#[derive(Debug, Serialize)]
pub struct UsuarioListItem {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub name: String,
    pub fecha_registro: DateTime<Utc>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
}

/// Candidato de Meet / Descubrir (alineado a `UserProfile` del front).
#[derive(Debug, Serialize, FromRow)]
pub struct DescubrirItem {
    pub id_usuario: i32,
    pub name: String,
    pub url_avatar: Option<String>,
    pub bio: Option<String>,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// El front envía `email`. También se acepta `username`.
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    /// Nombre completo del front (`fullName`).
    pub full_name: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub url_avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub url_avatar: Option<String>,
    pub full_name: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
    /// `publico` | `privado` | `solo_amigos` o etiquetas UI.
    pub privacidad: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsuarioListQuery {
    pub q: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DescubrirQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PasarDescubrirRequest {
    pub id_usuario: i32,
}

pub fn display_name(nombre: &Option<String>, username: &str) -> String {
    nombre
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(username)
        .to_string()
}

pub fn normalizar_privacidad(raw: &str) -> Option<&'static str> {
    match raw.trim().to_lowercase().as_str() {
        "publico" | "público" | "public" => Some("publico"),
        "privado" | "private" => Some("privado"),
        "solo_amigos" | "solo amigos" | "friends" | "friends_only" => Some("solo_amigos"),
        _ => None,
    }
}
