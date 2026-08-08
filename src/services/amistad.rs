use chrono::Utc;
use sqlx::PgPool;

use crate::models::amistad::{
    AmigoItem, CrearSolicitudRequest, SolicitudAmistad, SolicitudPendienteItem,
};
use crate::services::{chat, notificacion};

#[derive(Debug, thiserror::Error)]
pub enum AmistadError {
    #[error("no encontrado")]
    NotFound,

    #[error("prohibido")]
    Forbidden,

    #[error("conflicto: {0}")]
    Conflict(String),

    #[error("solicitud inválida: {0}")]
    InvalidRequest(String),

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

pub async fn enviar_solicitud(
    pool: &PgPool,
    id_remitente: i32,
    body: &CrearSolicitudRequest,
) -> Result<SolicitudAmistad, AmistadError> {
    let id_destinatario = body.id_usuario;
    if id_destinatario == id_remitente {
        return Err(AmistadError::InvalidRequest(
            "no puedes enviarte una solicitud a ti mismo".into(),
        ));
    }

    let existe: Option<i32> = sqlx::query_scalar(
        "SELECT id_usuario FROM usuarios WHERE id_usuario = $1",
    )
    .bind(id_destinatario)
    .fetch_optional(pool)
    .await?;

    if existe.is_none() {
        return Err(AmistadError::NotFound);
    }

    let existente = sqlx::query_as::<_, SolicitudAmistad>(
        r#"
        SELECT id_solicitud, id_remitente, id_destinatario, estado, fecha_creacion, fecha_respuesta
        FROM solicitudes_amistad
        WHERE (id_remitente = $1 AND id_destinatario = $2)
           OR (id_remitente = $2 AND id_destinatario = $1)
        ORDER BY fecha_creacion DESC
        LIMIT 1
        "#,
    )
    .bind(id_remitente)
    .bind(id_destinatario)
    .fetch_optional(pool)
    .await?;

    let solicitud = if let Some(s) = existente {
        match s.estado.as_str() {
            "aceptada" => {
                return Err(AmistadError::Conflict("ya son amigos".into()));
            }
            "pendiente" => {
                // Misma dirección: idempotente (evita error al reintentar / doble swipe).
                if s.id_remitente == id_remitente {
                    return Ok(s);
                }
                // Dirección inversa: ya te enviaron solicitud → aceptarla automáticamente.
                let (aceptada, _id_chat) =
                    aceptar(pool, id_remitente, s.id_solicitud).await?;
                return Ok(aceptada);
            }
            "rechazada" => {
                sqlx::query_as::<_, SolicitudAmistad>(
                    r#"
                    UPDATE solicitudes_amistad
                    SET id_remitente = $2,
                        id_destinatario = $3,
                        estado = 'pendiente',
                        fecha_creacion = NOW(),
                        fecha_respuesta = NULL
                    WHERE id_solicitud = $1
                    RETURNING id_solicitud, id_remitente, id_destinatario, estado,
                              fecha_creacion, fecha_respuesta
                    "#,
                )
                .bind(s.id_solicitud)
                .bind(id_remitente)
                .bind(id_destinatario)
                .fetch_one(pool)
                .await?
            }
            _ => {
                return Err(AmistadError::Conflict("estado inválido".into()));
            }
        }
    } else {
        sqlx::query_as::<_, SolicitudAmistad>(
            r#"
            INSERT INTO solicitudes_amistad (id_remitente, id_destinatario, estado)
            VALUES ($1, $2, 'pendiente')
            RETURNING id_solicitud, id_remitente, id_destinatario, estado,
                      fecha_creacion, fecha_respuesta
            "#,
        )
        .bind(id_remitente)
        .bind(id_destinatario)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db) = &e {
                if db.constraint().is_some() {
                    return AmistadError::Conflict("ya existe una solicitud".into());
                }
            }
            AmistadError::Database(e)
        })?
    };

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

    let _ = notificacion::crear(
        pool,
        id_destinatario,
        "solicitud_amistad",
        &format!("Nueva solicitud de amistad de {nombre_remitente}"),
        Some(solicitud.id_solicitud),
    )
    .await;

    Ok(solicitud)
}

pub async fn listar_pendientes(
    pool: &PgPool,
    id_destinatario: i32,
) -> Result<Vec<SolicitudPendienteItem>, AmistadError> {
    let rows = sqlx::query_as::<_, SolicitudPendienteItem>(
        r#"
        SELECT
            s.id_solicitud,
            s.id_remitente,
            COALESCE(NULLIF(btrim(u.nombre), ''), u.username) AS name,
            u.username,
            u.url_avatar,
            u.nombre,
            u.apellido,
            u.bio,
            'Te ha enviado una solicitud...' AS message,
            s.fecha_creacion
        FROM solicitudes_amistad s
        JOIN usuarios u ON u.id_usuario = s.id_remitente
        WHERE s.id_destinatario = $1 AND s.estado = 'pendiente'
        ORDER BY s.fecha_creacion ASC
        "#,
    )
    .bind(id_destinatario)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn aceptar(
    pool: &PgPool,
    id_destinatario: i32,
    id_solicitud: i32,
) -> Result<(SolicitudAmistad, i32), AmistadError> {
    let solicitud = obtener_pendiente_propia(pool, id_destinatario, id_solicitud).await?;

    let actualizada = sqlx::query_as::<_, SolicitudAmistad>(
        r#"
        UPDATE solicitudes_amistad
        SET estado = 'aceptada', fecha_respuesta = $2
        WHERE id_solicitud = $1
        RETURNING id_solicitud, id_remitente, id_destinatario, estado,
                  fecha_creacion, fecha_respuesta
        "#,
    )
    .bind(id_solicitud)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    let chat = chat::obtener_o_crear(
        pool,
        solicitud.id_remitente,
        solicitud.id_destinatario,
    )
    .await
    .map_err(|e| AmistadError::InvalidRequest(e.to_string()))?;

    let nombre_aceptador: String = sqlx::query_scalar(
        r#"
        SELECT COALESCE(NULLIF(btrim(nombre), ''), username)
        FROM usuarios WHERE id_usuario = $1
        "#,
    )
    .bind(id_destinatario)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|_| "Alguien".into());

    let _ = notificacion::crear(
        pool,
        solicitud.id_remitente,
        "solicitud_aceptada",
        &format!("{nombre_aceptador} aceptó tu solicitud de amistad"),
        Some(chat.id_chat),
    )
    .await;

    Ok((actualizada, chat.id_chat))
}

pub async fn rechazar(
    pool: &PgPool,
    id_destinatario: i32,
    id_solicitud: i32,
) -> Result<SolicitudAmistad, AmistadError> {
    let _ = obtener_pendiente_propia(pool, id_destinatario, id_solicitud).await?;

    let actualizada = sqlx::query_as::<_, SolicitudAmistad>(
        r#"
        UPDATE solicitudes_amistad
        SET estado = 'rechazada', fecha_respuesta = $2
        WHERE id_solicitud = $1
        RETURNING id_solicitud, id_remitente, id_destinatario, estado,
                  fecha_creacion, fecha_respuesta
        "#,
    )
    .bind(id_solicitud)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(actualizada)
}

async fn obtener_pendiente_propia(
    pool: &PgPool,
    id_destinatario: i32,
    id_solicitud: i32,
) -> Result<SolicitudAmistad, AmistadError> {
    let solicitud = sqlx::query_as::<_, SolicitudAmistad>(
        r#"
        SELECT id_solicitud, id_remitente, id_destinatario, estado, fecha_creacion, fecha_respuesta
        FROM solicitudes_amistad
        WHERE id_solicitud = $1
        "#,
    )
    .bind(id_solicitud)
    .fetch_optional(pool)
    .await?
    .ok_or(AmistadError::NotFound)?;

    if solicitud.id_destinatario != id_destinatario {
        return Err(AmistadError::Forbidden);
    }
    if solicitud.estado != "pendiente" {
        return Err(AmistadError::Conflict(
            "la solicitud ya fue respondida".into(),
        ));
    }

    Ok(solicitud)
}

pub async fn listar_amigos(
    pool: &PgPool,
    id_usuario: i32,
) -> Result<Vec<AmigoItem>, AmistadError> {
    let rows = sqlx::query_as::<_, AmigoItem>(
        r#"
        SELECT
            u.id_usuario,
            COALESCE(NULLIF(btrim(u.nombre), ''), u.username) AS name,
            u.username,
            u.url_avatar,
            u.nombre,
            u.apellido,
            u.bio,
            c.id_chat
        FROM solicitudes_amistad s
        JOIN usuarios u ON u.id_usuario = CASE
            WHEN s.id_remitente = $1 THEN s.id_destinatario
            ELSE s.id_remitente
        END
        LEFT JOIN chats c ON
            c.id_usuario_menor = LEAST($1, u.id_usuario)
            AND c.id_usuario_mayor = GREATEST($1, u.id_usuario)
        WHERE s.estado = 'aceptada'
          AND (s.id_remitente = $1 OR s.id_destinatario = $1)
        ORDER BY LOWER(COALESCE(u.nombre, u.username)) ASC
        "#,
    )
    .bind(id_usuario)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn son_amigos(pool: &PgPool, a: i32, b: i32) -> Result<bool, AmistadError> {
    let existe: Option<i32> = sqlx::query_scalar(
        r#"
        SELECT id_solicitud
        FROM solicitudes_amistad
        WHERE estado = 'aceptada'
          AND (
            (id_remitente = $1 AND id_destinatario = $2)
            OR (id_remitente = $2 AND id_destinatario = $1)
          )
        LIMIT 1
        "#,
    )
    .bind(a)
    .bind(b)
    .fetch_optional(pool)
    .await?;

    Ok(existe.is_some())
}
