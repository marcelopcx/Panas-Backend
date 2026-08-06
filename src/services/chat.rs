use std::collections::HashMap;
use std::sync::Mutex;

use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::models::chat::{
    Chat, ChatListItem, ChatParticipante, EnviarMensajeRequest, Mensaje,
};
use crate::services::{amistad, notificacion};

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

    sqlx::query_as::<_, Chat>(
        r#"
        INSERT INTO chats (id_usuario_menor, id_usuario_mayor)
        VALUES ($1, $2)
        RETURNING id_chat, id_usuario_menor, id_usuario_mayor, fecha_creacion
        "#,
    )
    .bind(menor)
    .bind(mayor)
    .fetch_one(pool)
    .await
    .map_err(ChatError::from)
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
        name: String,
        url_avatar: Option<String>,
        nombre: Option<String>,
        apellido: Option<String>,
        ultimo_contenido: Option<String>,
        ultimo_tipo: Option<String>,
        ultimo_fecha: Option<chrono::DateTime<chrono::Utc>>,
        unread: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            c.id_chat,
            c.fecha_creacion,
            u.id_usuario,
            u.username,
            COALESCE(NULLIF(btrim(u.nombre), ''), u.username) AS name,
            u.url_avatar,
            u.nombre,
            u.apellido,
            m.contenido AS ultimo_contenido,
            m.tipo AS ultimo_tipo,
            m.fecha_envio AS ultimo_fecha,
            (
              SELECT COUNT(*)::bigint
              FROM mensajes mx
              WHERE mx.id_chat = c.id_chat
                AND mx.id_remitente <> $1
                AND mx.fecha_envio > COALESCE(
                  CASE WHEN c.id_usuario_menor = $1 THEN c.ultima_lectura_menor
                       ELSE c.ultima_lectura_mayor END,
                  '1970-01-01'::timestamptz
                )
            ) AS unread
        FROM chats c
        JOIN usuarios u ON u.id_usuario = CASE
            WHEN c.id_usuario_menor = $1 THEN c.id_usuario_mayor
            ELSE c.id_usuario_menor
        END
        LEFT JOIN LATERAL (
            SELECT contenido, tipo, fecha_envio
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
        .map(|r| {
            let last_message = match r.ultimo_tipo.as_deref() {
                Some("imagen") => "📷 Imagen".to_string(),
                _ => r
                    .ultimo_contenido
                    .clone()
                    .unwrap_or_else(|| "Sin mensajes aún".into()),
            };
            let updated_at = r.ultimo_fecha.unwrap_or(r.fecha_creacion);

            ChatListItem {
                id_chat: r.id_chat,
                name: r.name.clone(),
                url_avatar: r.url_avatar.clone(),
                last_message,
                updated_at,
                unread: r.unread,
                otro_usuario: ChatParticipante {
                    id_usuario: r.id_usuario,
                    username: r.username,
                    name: r.name,
                    url_avatar: r.url_avatar,
                    nombre: r.nombre,
                    apellido: r.apellido,
                },
                fecha_creacion: r.fecha_creacion,
            }
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

pub async fn marcar_leido(
    pool: &PgPool,
    id_chat: i32,
    id_usuario: i32,
) -> Result<(), ChatError> {
    let chat = obtener_chat_si_participa(pool, id_chat, id_usuario).await?;

    if chat.id_usuario_menor == id_usuario {
        sqlx::query(
            "UPDATE chats SET ultima_lectura_menor = NOW() WHERE id_chat = $1",
        )
        .bind(id_chat)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE chats SET ultima_lectura_mayor = NOW() WHERE id_chat = $1",
        )
        .bind(id_chat)
        .execute(pool)
        .await?;
    }

    Ok(())
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

    // Al abrir el historial, marcar como leído
    let _ = marcar_leido(pool, id_chat, id_usuario).await;

    sqlx::query_as::<_, Mensaje>(
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
    .await
    .map_err(ChatError::from)
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
        .or(body.text.as_deref())
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
            "debes enviar contenido/text o url_imagen".into(),
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

    // Actualizar lectura del remitente
    let _ = marcar_leido(pool, id_chat, id_remitente).await;

    let nombre_remitente: String = sqlx::query_scalar(
        r#"
        SELECT COALESCE(NULLIF(btrim(nombre), ''), username)
        FROM usuarios WHERE id_usuario = $1
        "#,
    )
    .bind(id_remitente)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|_| "Alguien".into());

    let preview = if tipo == "imagen" {
        format!("Nuevo mensaje de {nombre_remitente}: 📷 Imagen")
    } else {
        let texto = contenido.as_deref().unwrap_or("");
        let corto: String = texto.chars().take(60).collect();
        format!("Nuevo mensaje de {nombre_remitente}: {corto}")
    };

    let _ = notificacion::crear(
        pool,
        otro,
        "mensaje",
        &preview,
        Some(id_chat),
    )
    .await;

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
