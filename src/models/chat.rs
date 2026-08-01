use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Chat {
    pub id_chat: i32,
    pub id_usuario_menor: i32,
    pub id_usuario_mayor: i32,
    pub fecha_creacion: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ChatListItem {
    pub id_chat: i32,
    pub otro_usuario: ChatParticipante,
    pub ultimo_mensaje: Option<MensajeResumen>,
    pub fecha_creacion: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ChatParticipante {
    pub id_usuario: i32,
    pub username: String,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mensaje {
    pub id_mensaje: i32,
    pub id_chat: i32,
    pub id_remitente: i32,
    pub contenido: Option<String>,
    pub url_imagen: Option<String>,
    pub tipo: String,
    pub fecha_envio: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MensajeResumen {
    pub id_mensaje: i32,
    pub id_remitente: i32,
    pub contenido: Option<String>,
    pub url_imagen: Option<String>,
    pub tipo: String,
    pub fecha_envio: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EnviarMensajeRequest {
    pub contenido: Option<String>,
    pub url_imagen: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MensajesQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// Eventos enviados/recibidos por WebSocket (JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Mensaje {
        mensaje: Mensaje,
    },
    Error {
        error: String,
    },
    Ping,
    Pong,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Enviar {
        contenido: Option<String>,
        url_imagen: Option<String>,
    },
    Ping,
}
