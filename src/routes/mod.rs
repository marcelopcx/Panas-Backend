use actix_web::web;

use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::health_check)
        // Auth / perfil
        .service(handlers::register)
        .service(handlers::login)
        .service(handlers::forgot_password)
        .service(handlers::get_me)
        .service(handlers::patch_me)
        .service(handlers::delete_me)
        .service(handlers::subir_avatar_usuario)
        .service(handlers::listar_usuarios)
        .service(handlers::get_usuario)
        // Meet / descubrir (swipe)
        .service(handlers::listar_descubrir)
        .service(handlers::pasar_descubrir)
        // Amistades (bandeja)
        .service(handlers::crear_solicitud)
        .service(handlers::listar_pendientes)
        .service(handlers::listar_amigos)
        .service(handlers::aceptar_amistad)
        .service(handlers::rechazar_amistad)
        .service(handlers::decidir_amistad)
        // Chats + mensajes
        .service(handlers::listar_chats)
        .service(handlers::abrir_chat)
        .service(handlers::listar_mensajes)
        .service(handlers::enviar_mensaje)
        .service(handlers::marcar_chat_leido)
        .service(handlers::subir_imagen_chat)
        .service(handlers::subir_imagen_generica)
        // Notificaciones (campana)
        .service(handlers::listar_notificaciones)
        .service(handlers::marcar_notificacion_leida)
        .service(handlers::marcar_todas_leidas)
        .service(handlers::eliminar_notificacion)
        // WebSocket
        .route("/ws/chats/{id}", web::get().to(handlers::ws::ws_chat));
}
