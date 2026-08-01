//! WebSocket de chat en vivo.
//!
//! Conexión: `GET /ws/chats/{id}?token=<JWT>`
//!
//! Cliente → servidor (JSON):
//! ```json
//! { "type": "enviar", "contenido": "hola", "url_imagen": null }
//! { "type": "ping" }
//! ```
//!
//! Servidor → cliente (JSON):
//! ```json
//! { "type": "mensaje", "mensaje": { ... } }
//! { "type": "pong" }
//! { "type": "error", "error": "..." }
//! ```

use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt as _;
use sqlx::PgPool;

use crate::config::AppConfig;
use crate::error::ApiError;
use crate::models::chat::{EnviarMensajeRequest, WsClientMessage, WsEvent};
use crate::services::auth;
use crate::services::chat::{self, ChatHub};

pub async fn ws_chat(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
    hub: web::Data<ChatHub>,
    config: web::Data<AppConfig>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id_chat = path.into_inner();

    let token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| {
            req.uri()
                .query()
                .and_then(|q| {
                    q.split('&').find_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        match (parts.next(), parts.next()) {
                            (Some("token"), Some(value)) => Some(value),
                            _ => None,
                        }
                    })
                })
        })
        .ok_or(ApiError::NoAutorizado)?;

    let user_id = auth::user_id_from_token(token, &config.jwt_secret)
        .map_err(|_| ApiError::NoAutorizado)?;

    // Validar participación antes del upgrade WS
    chat::obtener_chat_si_participa(pool.get_ref(), id_chat, user_id).await?;

    let (response, mut session, msg_stream) =
        actix_ws::handle(&req, stream).map_err(|e| {
            ApiError::ErrorDelServidor(format!("websocket: {e}"))
        })?;

    let mut msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let mut rx = hub.subscribe(id_chat);
    let hub_clone = hub.clone();
    let pool_clone = pool.clone();

    actix_web::rt::spawn(async move {
        // Reenviar broadcasts del hub al cliente
        let mut session_out = session.clone();
        actix_web::rt::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(payload) => {
                        if session_out.text(payload).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                actix_ws::AggregatedMessage::Text(text) => {
                    match serde_json::from_str::<WsClientMessage>(&text) {
                        Ok(WsClientMessage::Ping) => {
                            let _ = session
                                .text(
                                    serde_json::to_string(&WsEvent::Pong)
                                        .unwrap_or_else(|_| r#"{"type":"pong"}"#.into()),
                                )
                                .await;
                        }
                        Ok(WsClientMessage::Enviar {
                            contenido,
                            url_imagen,
                        }) => {
                            match chat::enviar_mensaje(
                                pool_clone.get_ref(),
                                id_chat,
                                user_id,
                                &EnviarMensajeRequest {
                                    contenido,
                                    url_imagen,
                                },
                            )
                            .await
                            {
                                Ok(mensaje) => {
                                    if let Ok(payload) =
                                        serde_json::to_string(&WsEvent::Mensaje { mensaje })
                                    {
                                        hub_clone.publish(id_chat, payload);
                                    }
                                }
                                Err(e) => {
                                    let _ = session
                                        .text(
                                            serde_json::to_string(&WsEvent::Error {
                                                error: e.to_string(),
                                            })
                                            .unwrap_or_default(),
                                        )
                                        .await;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = session
                                .text(
                                    serde_json::to_string(&WsEvent::Error {
                                        error: format!("JSON inválido: {e}"),
                                    })
                                    .unwrap_or_default(),
                                )
                                .await;
                        }
                    }
                }
                actix_ws::AggregatedMessage::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                actix_ws::AggregatedMessage::Close(_) => break,
                _ => {}
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
