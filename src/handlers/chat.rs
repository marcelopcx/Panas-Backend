use actix_web::{get, post, web, HttpResponse};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::models::chat::{EnviarMensajeRequest, MensajesQuery, WsEvent};
use crate::services::chat::{self, ChatHub};

#[derive(serde::Deserialize)]
pub struct AbrirChatRequest {
    pub id_usuario: i32,
}

/// Lista chats del usuario autenticado.
#[get("/chats")]
pub async fn listar_chats(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let chats = chat::listar_chats(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(chats))
}

/// Abre (o reutiliza) un chat 1:1 con un amigo.
#[post("/chats")]
pub async fn abrir_chat(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    body: web::Json<AbrirChatRequest>,
) -> Result<HttpResponse, ApiError> {
    let chat =
        chat::abrir_chat_con_amigo(pool.get_ref(), user.user_id, body.id_usuario).await?;
    Ok(HttpResponse::Ok().json(chat))
}

/// Historial paginado de mensajes (más recientes primero).
#[get("/chats/{id}/mensajes")]
pub async fn listar_mensajes(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    query: web::Query<MensajesQuery>,
) -> Result<HttpResponse, ApiError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);
    let mensajes =
        chat::listar_mensajes(pool.get_ref(), path.into_inner(), user.user_id, page, limit)
            .await?;
    Ok(HttpResponse::Ok().json(mensajes))
}

/// Envía un mensaje por REST (también se emite por WebSocket si hay suscriptores).
#[post("/chats/{id}/mensajes")]
pub async fn enviar_mensaje(
    pool: web::Data<PgPool>,
    hub: web::Data<ChatHub>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    body: web::Json<EnviarMensajeRequest>,
) -> Result<HttpResponse, ApiError> {
    let id_chat = path.into_inner();
    let mensaje =
        chat::enviar_mensaje(pool.get_ref(), id_chat, user.user_id, &body).await?;

    if let Ok(payload) = serde_json::to_string(&WsEvent::Mensaje {
        mensaje: mensaje.clone(),
    }) {
        hub.publish(id_chat, payload);
    }

    Ok(HttpResponse::Created().json(mensaje))
}
