use chrono::Utc;
use sqlx::PgPool;

use crate::models::amistad::{
    AmigoItem, CrearSolicitudRequest, SolicitudAmistad, SolicitudPendienteItem,
};
use crate::services::chat;

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

    // ¿Ya son amigos o hay solicitud en cualquier dirección?
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

    if let Some(s) = existente {
        match s.estado.as_str() {
            "aceptada" => {
                return Err(AmistadError::Conflict("ya son amigos".into()));
            }
            "pendiente" => {
                return Err(AmistadError::Conflict(
                    "ya existe una solicitud pendiente".into(),
                ));
            }
            "rechazada" => {
                // Reabrir como nueva solicitud del remitente actual
                let renovada = sqlx::query_as::<_, SolicitudAmistad>(
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
                .await?;
                return Ok(renovada);
            }
            _ => {}
        }
    }

    let solicitud = sqlx::query_as::<_, SolicitudAmistad>(
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
    })?;

    Ok(solicitud)
}

/// Solicitudes pendientes recibidas (para swipe izquierda/derecha).
pub async fn listar_pendientes(
    pool: &PgPool,
    id_destinatario: i32,
) -> Result<Vec<SolicitudPendienteItem>, AmistadError> {
    let rows = sqlx::query_as::<_, SolicitudPendienteItem>(
        r#"
        SELECT
            s.id_solicitud,
            s.id_remitente,
            u.username,
            u.url_avatar,
            u.nombre,
            u.apellido,
            u.bio,
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
        ORDER BY LOWER(u.username) ASC
        "#,
    )
    .bind(id_usuario)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn son_amigos(
    pool: &PgPool,
    a: i32,
    b: i32,
) -> Result<bool, AmistadError> {
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
