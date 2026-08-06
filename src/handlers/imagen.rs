//! Subida de imágenes a Cloudinary (avatares y chat).

use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse};
use futures_util::StreamExt;
use serde::Serialize;
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::config::AppConfig;
use crate::error::ApiError;
use crate::models::chat::EnviarMensajeRequest;
use crate::services::{auth, chat, cloudinary};
use crate::services::chat::ChatHub;

const MAX_BYTES: usize = 10 * 1024 * 1024;

#[derive(Serialize)]
struct ImagenSubidaResponse {
    secure_url: String,
}

pub async fn leer_archivo_multipart(payload: &mut Multipart) -> Result<(Vec<u8>, String), ApiError> {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            ApiError::SolicitudInvalida(format!("multipart inválido: {e}"))
        })?;

        if field.name() != Some("file") {
            continue;
        }

        let filename = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .map(|name| name.to_string())
            .unwrap_or_else(|| "imagen.jpg".to_string());

        let mut bytes: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| {
                ApiError::ErrorDelServidor(format!("error leyendo archivo: {e}"))
            })?;

            if bytes.len() + data.len() > MAX_BYTES {
                return Err(ApiError::SolicitudInvalida(
                    "la imagen supera el tamaño máximo permitido (10 MB)".into(),
                ));
            }

            bytes.extend_from_slice(&data);
        }

        if bytes.is_empty() {
            return Err(ApiError::SolicitudInvalida("el archivo está vacío".into()));
        }

        return Ok((bytes, filename));
    }

    Err(ApiError::SolicitudInvalida(
        "no se envió ningún archivo en el campo `file`".into(),
    ))
}

/// Sube avatar a Cloudinary y actualiza el perfil del usuario autenticado.
#[post("/auth/me/avatar")]
pub async fn subir_avatar_usuario(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    user: AuthenticatedUser,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let (bytes, filename) = leer_archivo_multipart(&mut payload).await?;
    let folder = format!("{}/avatars", config.cloudinary.folder);
    let secure_url =
        cloudinary::subir_imagen(&config.cloudinary, bytes, filename, Some(&folder)).await?;

    let perfil = auth::actualizar_avatar(pool.get_ref(), user.user_id, &secure_url).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "secure_url": secure_url,
        "user": perfil
    })))
}

/// Sube una imagen de chat, la persiste como mensaje y la emite por WebSocket.
#[post("/chats/{id}/imagen")]
pub async fn subir_imagen_chat(
    pool: web::Data<PgPool>,
    hub: web::Data<ChatHub>,
    config: web::Data<AppConfig>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let id_chat = path.into_inner();
    let (bytes, filename) = leer_archivo_multipart(&mut payload).await?;
    let folder = format!("{}/chats", config.cloudinary.folder);
    let secure_url =
        cloudinary::subir_imagen(&config.cloudinary, bytes, filename, Some(&folder)).await?;

    let mensaje = chat::enviar_mensaje(
        pool.get_ref(),
        id_chat,
        user.user_id,
        &EnviarMensajeRequest {
            contenido: None,
            url_imagen: Some(secure_url.clone()),
            text: None,
        },
    )
    .await?;

    if let Ok(payload_ws) = serde_json::to_string(&crate::models::chat::WsEvent::Mensaje {
        mensaje: mensaje.clone(),
    }) {
        hub.publish(id_chat, payload_ws);
    }

    Ok(HttpResponse::Created().json(serde_json::json!({
        "secure_url": secure_url,
        "mensaje": mensaje
    })))
}

#[post("/uploads/imagen")]
pub async fn subir_imagen_generica(
    config: web::Data<AppConfig>,
    _user: AuthenticatedUser,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let (bytes, filename) = leer_archivo_multipart(&mut payload).await?;
    let folder = format!("{}/misc", config.cloudinary.folder);
    let secure_url =
        cloudinary::subir_imagen(&config.cloudinary, bytes, filename, Some(&folder)).await?;

    Ok(HttpResponse::Ok().json(ImagenSubidaResponse { secure_url }))
}
