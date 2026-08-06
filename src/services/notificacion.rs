use sqlx::PgPool;

use crate::models::notificacion::{Notificacion, NotificacionesQuery};

#[derive(Debug, thiserror::Error)]
pub enum NotificacionError {
    #[error("no encontrado")]
    NotFound,

    #[error("prohibido")]
    Forbidden,

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

pub async fn crear(
    pool: &PgPool,
    id_usuario: i32,
    tipo: &str,
    mensaje: &str,
    id_referencia: Option<i32>,
) -> Result<Notificacion, NotificacionError> {
    let row = sqlx::query_as::<_, Notificacion>(
        r#"
        INSERT INTO notificaciones (id_usuario, tipo, mensaje, id_referencia)
        VALUES ($1, $2, $3, $4)
        RETURNING id_notificacion, id_usuario, tipo, mensaje, leida, id_referencia, fecha_creacion
        "#,
    )
    .bind(id_usuario)
    .bind(tipo)
    .bind(mensaje)
    .bind(id_referencia)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn listar(
    pool: &PgPool,
    id_usuario: i32,
    query: &NotificacionesQuery,
) -> Result<Vec<Notificacion>, NotificacionError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let solo_no_leidas = query.solo_no_leidas.unwrap_or(false);

    let rows = sqlx::query_as::<_, Notificacion>(
        r#"
        SELECT id_notificacion, id_usuario, tipo, mensaje, leida, id_referencia, fecha_creacion
        FROM notificaciones
        WHERE id_usuario = $1
          AND ($2::bool = FALSE OR leida = FALSE)
        ORDER BY fecha_creacion DESC
        LIMIT $3
        "#,
    )
    .bind(id_usuario)
    .bind(solo_no_leidas)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn marcar_leida(
    pool: &PgPool,
    id_usuario: i32,
    id_notificacion: i32,
) -> Result<Notificacion, NotificacionError> {
    let row = sqlx::query_as::<_, Notificacion>(
        r#"
        UPDATE notificaciones
        SET leida = TRUE
        WHERE id_notificacion = $1 AND id_usuario = $2
        RETURNING id_notificacion, id_usuario, tipo, mensaje, leida, id_referencia, fecha_creacion
        "#,
    )
    .bind(id_notificacion)
    .bind(id_usuario)
    .fetch_optional(pool)
    .await?
    .ok_or(NotificacionError::NotFound)?;

    Ok(row)
}

pub async fn marcar_todas_leidas(
    pool: &PgPool,
    id_usuario: i32,
) -> Result<u64, NotificacionError> {
    let result = sqlx::query(
        r#"
        UPDATE notificaciones SET leida = TRUE
        WHERE id_usuario = $1 AND leida = FALSE
        "#,
    )
    .bind(id_usuario)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn eliminar(
    pool: &PgPool,
    id_usuario: i32,
    id_notificacion: i32,
) -> Result<(), NotificacionError> {
    let result = sqlx::query(
        r#"
        DELETE FROM notificaciones
        WHERE id_notificacion = $1 AND id_usuario = $2
        "#,
    )
    .bind(id_notificacion)
    .bind(id_usuario)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(NotificacionError::NotFound);
    }

    Ok(())
}

pub async fn contar_no_leidas(pool: &PgPool, id_usuario: i32) -> Result<i64, NotificacionError> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM notificaciones
        WHERE id_usuario = $1 AND leida = FALSE
        "#,
    )
    .bind(id_usuario)
    .fetch_one(pool)
    .await?;

    Ok(count)
}
