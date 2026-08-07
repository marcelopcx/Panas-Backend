use actix_web::{get, post, web, HttpResponse};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::models::chat::{EnviarMensajeRequest, MensajesQuery};
use crate::services::chat::{self, ChatHub};

#[derive(serde::Deserialize)]
pub struct AbrirChatRequest {
    pub id_usuario: i32,
}

#[get("/chats")]
pub async fn listar_chats(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let chats = chat::listar_chats(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(chats))
}

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

#[post("/chats/{id}/mensajes")]
pub async fn enviar_mensaje(
    pool: web::Data<PgPool>,
    hub: web::Data<ChatHub>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    body: web::Json<EnviarMensajeRequest>,
) -> Result<HttpResponse, ApiError> {
    let id_chat = path.into_inner();
    let (mensaje, chat) =
        chat::enviar_mensaje(pool.get_ref(), id_chat, user.user_id, &body).await?;

    hub.emit_mensaje(&chat, &mensaje);

    Ok(HttpResponse::Created().json(mensaje))
}

/// Marca el chat como leído (pone unread en 0).
#[post("/chats/{id}/leer")]
pub async fn marcar_chat_leido(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    chat::marcar_leido(pool.get_ref(), path.into_inner(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "ok": true })))
}
