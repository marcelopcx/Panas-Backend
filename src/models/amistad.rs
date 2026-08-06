use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SolicitudAmistad {
    pub id_solicitud: i32,
    pub id_remitente: i32,
    pub id_destinatario: i32,
    pub estado: String,
    pub fecha_creacion: DateTime<Utc>,
    pub fecha_respuesta: Option<DateTime<Utc>>,
}

/// Item de bandeja (Inbox) — alineado a `FriendRequestItem`.
#[derive(Debug, Serialize, FromRow)]
pub struct SolicitudPendienteItem {
    pub id_solicitud: i32,
    pub id_remitente: i32,
    pub name: String,
    pub username: String,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
    pub message: String,
    pub fecha_creacion: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AmigoItem {
    pub id_usuario: i32,
    pub name: String,
    pub username: String,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub bio: Option<String>,
    pub id_chat: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CrearSolicitudRequest {
    pub id_usuario: i32,
}

/// Acción de swipe en bandeja: `aceptar` (derecha) o `rechazar` (izquierda).
#[derive(Debug, Deserialize)]
pub struct DecidirAmistadRequest {
    pub accion: String,
}
