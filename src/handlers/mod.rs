pub mod amistad;
pub mod auth;
pub mod chat;
pub mod health;
pub mod imagen;
pub mod notificacion;
pub mod ws;

pub use amistad::{
    aceptar_amistad, crear_solicitud, decidir_amistad, listar_amigos, listar_pendientes,
    rechazar_amistad,
};
pub use auth::{
    delete_me, forgot_password, get_me, get_usuario, listar_descubrir, listar_usuarios, login,
    pasar_descubrir, patch_me, register,
};
pub use chat::{abrir_chat, enviar_mensaje, listar_chats, listar_mensajes, marcar_chat_leido};
pub use health::health_check;
pub use imagen::{subir_avatar_usuario, subir_imagen_chat, subir_imagen_generica};
pub use notificacion::{
    eliminar_notificacion, listar_notificaciones, marcar_notificacion_leida, marcar_todas_leidas,
};
