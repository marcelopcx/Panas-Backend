pub mod amistad;
pub mod auth;
pub mod chat;
pub mod health;
pub mod imagen;
pub mod ws;

pub use amistad::{
    aceptar_amistad, crear_solicitud, decidir_amistad, listar_amigos, listar_pendientes,
    rechazar_amistad,
};
pub use auth::{delete_me, get_me, get_usuario, listar_usuarios, login, patch_me, register};
pub use chat::{abrir_chat, enviar_mensaje, listar_chats, listar_mensajes};
pub use health::health_check;
pub use imagen::{subir_avatar_usuario, subir_imagen_chat, subir_imagen_generica};
