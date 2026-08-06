use actix_web::{delete, get, patch, post, web, HttpResponse};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::models::notificacion::NotificacionesQuery;
use crate::services::notificacion;

#[get("/notificaciones")]
pub async fn listar_notificaciones(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    query: web::Query<NotificacionesQuery>,
) -> Result<HttpResponse, ApiError> {
    let items = notificacion::listar(pool.get_ref(), user.user_id, &query).await?;
    let unread = notificacion::contar_no_leidas(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": items,
        "unread": unread
    })))
}

#[patch("/notificaciones/{id}/leer")]
pub async fn marcar_notificacion_leida(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let item =
        notificacion::marcar_leida(pool.get_ref(), user.user_id, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(item))
}

#[post("/notificaciones/leer-todas")]
pub async fn marcar_todas_leidas(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let updated = notificacion::marcar_todas_leidas(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "updated": updated })))
}

#[delete("/notificaciones/{id}")]
pub async fn eliminar_notificacion(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    notificacion::eliminar(pool.get_ref(), user.user_id, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
