use actix_web::{get, post, web, HttpResponse};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::models::amistad::{CrearSolicitudRequest, DecidirAmistadRequest};
use crate::services::amistad;

/// Envía una solicitud de amistad.
#[post("/amistades")]
pub async fn crear_solicitud(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    body: web::Json<CrearSolicitudRequest>,
) -> Result<HttpResponse, ApiError> {
    let solicitud = amistad::enviar_solicitud(pool.get_ref(), user.user_id, &body).await?;
    Ok(HttpResponse::Created().json(solicitud))
}

/// Lista solicitudes pendientes recibidas (para swipe).
#[get("/amistades/pendientes")]
pub async fn listar_pendientes(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let pendientes = amistad::listar_pendientes(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(pendientes))
}

/// Lista amigos aceptados (incluye `id_chat` si existe).
#[get("/amistades")]
pub async fn listar_amigos(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let amigos = amistad::listar_amigos(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(amigos))
}

/// Swipe derecha: aceptar amistad (crea chat 1:1).
#[post("/amistades/{id}/aceptar")]
pub async fn aceptar_amistad(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let (solicitud, id_chat) =
        amistad::aceptar(pool.get_ref(), user.user_id, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "solicitud": solicitud,
        "id_chat": id_chat,
        "accion": "aceptar"
    })))
}

/// Swipe izquierda: rechazar amistad.
#[post("/amistades/{id}/rechazar")]
pub async fn rechazar_amistad(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let solicitud =
        amistad::rechazar(pool.get_ref(), user.user_id, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "solicitud": solicitud,
        "accion": "rechazar"
    })))
}

/// Endpoint unificado para swipe: `{ "accion": "aceptar" | "rechazar" }`.
#[post("/amistades/{id}/decidir")]
pub async fn decidir_amistad(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    body: web::Json<DecidirAmistadRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    match body.accion.trim().to_lowercase().as_str() {
        "aceptar" | "accept" | "right" | "derecha" => {
            let (solicitud, id_chat) = amistad::aceptar(pool.get_ref(), user.user_id, id).await?;
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "solicitud": solicitud,
                "id_chat": id_chat,
                "accion": "aceptar"
            })))
        }
        "rechazar" | "reject" | "left" | "izquierda" => {
            let solicitud = amistad::rechazar(pool.get_ref(), user.user_id, id).await?;
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "solicitud": solicitud,
                "accion": "rechazar"
            })))
        }
        _ => Err(ApiError::SolicitudInvalida(
            "accion debe ser 'aceptar' (derecha) o 'rechazar' (izquierda)".into(),
        )),
    }
}
