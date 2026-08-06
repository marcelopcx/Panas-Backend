use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Notificacion {
    pub id_notificacion: i32,
    pub id_usuario: i32,
    pub tipo: String,
    pub mensaje: String,
    pub leida: bool,
    pub id_referencia: Option<i32>,
    pub fecha_creacion: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NotificacionesQuery {
    pub solo_no_leidas: Option<bool>,
    pub limit: Option<i64>,
}
