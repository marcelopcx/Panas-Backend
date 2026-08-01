use std::collections::HashMap;
use std::sync::Mutex;

use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::models::chat::{
    Chat, ChatListItem, ChatParticipante, EnviarMensajeRequest, Mensaje, MensajeResumen,
};
use crate::services::amistad;

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("no encontrado")]
    NotFound,

    #[error("prohibido")]
    Forbidden,

    #[error("solicitud inválida: {0}")]
    InvalidRequest(String),

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

/// Hub en memoria: un canal broadcast por chat para WebSockets.
#[derive(Default)]
pub struct ChatHub {
    inner: Mutex<HashMap<i32, broadcast::Sender<String>>>,
}

impl ChatHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, id_chat: i32) -> broadcast::Receiver<String> {
        let mut map = self.inner.lock().expect("chat hub lock");
        map.entry(id_chat)
            .or_insert_with(|| broadcast::channel(256).0)
            .subscribe()
    }

    pub fn publish(&self, id_chat: i32, payload: String) {
        let mut map = self.inner.lock().expect("chat hub lock");
        let sender = map
            .entry(id_chat)
            .or_insert_with(|| broadcast::channel(256).0);
        let _ = sender.send(payload);
    }
}

pub async fn obtener_o_crear(
    pool: &PgPool,
    usuario_a: i32,
    usuario_b: i32,
) -> Result<Chat, ChatError> {
    let menor = usuario_a.min(usuario_b);
    let mayor = usuario_a.max(usuario_b);

    if let Some(chat) = sqlx::query_as::<_, Chat>(
        r#"
        SELECT id_chat, id_usuario_menor, id_usuario_mayor, fecha_creacion
        FROM chats
        WHERE id_usuario_menor = $1 AND id_usuario_mayor = $2
        "#,
    )
    .bind(menor)
    .bind(mayor)
    .fetch_optional(pool)
    .await?
    {
        return Ok(chat);
    }

    let chat = sqlx::query_as::<_, Chat>(
        r#"
        INSERT INTO chats (id_usuario_menor, id_usuario_mayor)
        VALUES ($1, $2)
        RETURNING id_chat, id_usuario_menor, id_usuario_mayor, fecha_creacion
        "#,
    )
    .bind(menor)
    .bind(mayor)
    .fetch_one(pool)
    .await?;

    Ok(chat)
}

pub async fn listar_chats(
    pool: &PgPool,
    id_usuario: i32,
) -> Result<Vec<ChatListItem>, ChatError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id_chat: i32,
        fecha_creacion: chrono::DateTime<chrono::Utc>,
        id_usuario: i32,
        username: String,
        url_avatar: Option<String>,
        nombre: Option<String>,
        apellido: Option<String>,
        ultimo_id: Option<i32>,
        ultimo_remitente: Option<i32>,
        ultimo_contenido: Option<String>,
        ultimo_url_imagen: Option<String>,
        ultimo_tipo: Option<String>,
        ultimo_fecha: Option<chrono::DateTime<chrono::Utc>>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            c.id_chat,
            c.fecha_creacion,
            u.id_usuario,
            u.username,
            u.url_avatar,
            u.nombre,
            u.apellido,
            m.id_mensaje AS ultimo_id,
            m.id_remitente AS ultimo_remitente,
            m.contenido AS ultimo_contenido,
            m.url_imagen AS ultimo_url_imagen,
            m.tipo AS ultimo_tipo,
            m.fecha_envio AS ultimo_fecha
        FROM chats c
        JOIN usuarios u ON u.id_usuario = CASE
            WHEN c.id_usuario_menor = $1 THEN c.id_usuario_mayor
            ELSE c.id_usuario_menor
        END
        LEFT JOIN LATERAL (
            SELECT id_mensaje, id_remitente, contenido, url_imagen, tipo, fecha_envio
            FROM mensajes
            WHERE id_chat = c.id_chat
            ORDER BY fecha_envio DESC
            LIMIT 1
        ) m ON TRUE
        WHERE c.id_usuario_menor = $1 OR c.id_usuario_mayor = $1
        ORDER BY COALESCE(m.fecha_envio, c.fecha_creacion) DESC
        "#,
    )
    .bind(id_usuario)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ChatListItem {
            id_chat: r.id_chat,
            otro_usuario: ChatParticipante {
                id_usuario: r.id_usuario,
                username: r.username,
                url_avatar: r.url_avatar,
                nombre: r.nombre,
                apellido: r.apellido,
            },
            ultimo_mensaje: match (r.ultimo_id, r.ultimo_remitente, r.ultimo_tipo, r.ultimo_fecha)
            {
                (Some(id), Some(rem), Some(tipo), Some(fecha)) => Some(MensajeResumen {
                    id_mensaje: id,
                    id_remitente: rem,
                    contenido: r.ultimo_contenido,
                    url_imagen: r.ultimo_url_imagen,
                    tipo,
                    fecha_envio: fecha,
                }),
                _ => None,
            },
            fecha_creacion: r.fecha_creacion,
        })
        .collect())
}

pub async fn obtener_chat_si_participa(
    pool: &PgPool,
    id_chat: i32,
    id_usuario: i32,
) -> Result<Chat, ChatError> {
    let chat = sqlx::query_as::<_, Chat>(
        r#"
        SELECT id_chat, id_usuario_menor, id_usuario_mayor, fecha_creacion
        FROM chats
        WHERE id_chat = $1
        "#,
    )
    .bind(id_chat)
    .fetch_optional(pool)
    .await?
    .ok_or(ChatError::NotFound)?;

    if chat.id_usuario_menor != id_usuario && chat.id_usuario_mayor != id_usuario {
        return Err(ChatError::Forbidden);
    }

    Ok(chat)
}

pub async fn listar_mensajes(
    pool: &PgPool,
    id_chat: i32,
    id_usuario: i32,
    page: i64,
    limit: i64,
) -> Result<Vec<Mensaje>, ChatError> {
    let _ = obtener_chat_si_participa(pool, id_chat, id_usuario).await?;
    let page = page.max(1);
    let limit = limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    let mensajes = sqlx::query_as::<_, Mensaje>(
        r#"
        SELECT id_mensaje, id_chat, id_remitente, contenido, url_imagen, tipo, fecha_envio
        FROM mensajes
        WHERE id_chat = $1
        ORDER BY fecha_envio DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(id_chat)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(mensajes)
}

pub async fn enviar_mensaje(
    pool: &PgPool,
    id_chat: i32,
    id_remitente: i32,
    body: &EnviarMensajeRequest,
) -> Result<Mensaje, ChatError> {
    let chat = obtener_chat_si_participa(pool, id_chat, id_remitente).await?;

    let otro = if chat.id_usuario_menor == id_remitente {
        chat.id_usuario_mayor
    } else {
        chat.id_usuario_menor
    };

    if !amistad::son_amigos(pool, id_remitente, otro)
        .await
        .map_err(|e| ChatError::InvalidRequest(e.to_string()))?
    {
        return Err(ChatError::Forbidden);
    }

    let contenido = body
        .contenido
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let url_imagen = body
        .url_imagen
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if contenido.is_none() && url_imagen.is_none() {
        return Err(ChatError::InvalidRequest(
            "debes enviar contenido o url_imagen".into(),
        ));
    }

    let tipo = if url_imagen.is_some() {
        "imagen"
    } else {
        "texto"
    };

    let mensaje = sqlx::query_as::<_, Mensaje>(
        r#"
        INSERT INTO mensajes (id_chat, id_remitente, contenido, url_imagen, tipo)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id_mensaje, id_chat, id_remitente, contenido, url_imagen, tipo, fecha_envio
        "#,
    )
    .bind(id_chat)
    .bind(id_remitente)
    .bind(&contenido)
    .bind(&url_imagen)
    .bind(tipo)
    .fetch_one(pool)
    .await?;

    Ok(mensaje)
}

pub async fn abrir_chat_con_amigo(
    pool: &PgPool,
    id_usuario: i32,
    id_amigo: i32,
) -> Result<Chat, ChatError> {
    if id_usuario == id_amigo {
        return Err(ChatError::InvalidRequest(
            "no puedes abrir un chat contigo mismo".into(),
        ));
    }

    if !amistad::son_amigos(pool, id_usuario, id_amigo)
        .await
        .map_err(|e| ChatError::InvalidRequest(e.to_string()))?
    {
        return Err(ChatError::Forbidden);
    }

    obtener_o_crear(pool, id_usuario, id_amigo).await
}
