use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;

use crate::services::auth::AuthError;

#[derive(Debug)]
pub enum ApiError {
    LoginIncorrecto,
    NoAutorizado,
    Prohibido,
    NoEncontrado,
    UsuarioYaExiste,
    SolicitudInvalida(String),
    ErrorDelServidor(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mensaje())
    }
}

impl ApiError {
    fn mensaje(&self) -> String {
        match self {
            ApiError::LoginIncorrecto => "credenciales inválidas".into(),
            ApiError::NoAutorizado => "no autorizado".into(),
            ApiError::Prohibido => "no tienes permiso para esta acción".into(),
            ApiError::NoEncontrado => "recurso no encontrado".into(),
            ApiError::UsuarioYaExiste => "el usuario ya existe".into(),
            ApiError::SolicitudInvalida(detalle) => detalle.clone(),
            ApiError::ErrorDelServidor(detalle) => detalle.clone(),
        }
    }

    fn codigo_http(&self) -> StatusCode {
        match self {
            ApiError::LoginIncorrecto => StatusCode::UNAUTHORIZED,
            ApiError::NoAutorizado => StatusCode::UNAUTHORIZED,
            ApiError::Prohibido => StatusCode::FORBIDDEN,
            ApiError::NoEncontrado => StatusCode::NOT_FOUND,
            ApiError::UsuarioYaExiste => StatusCode::CONFLICT,
            ApiError::SolicitudInvalida(_) => StatusCode::BAD_REQUEST,
            ApiError::ErrorDelServidor(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => ApiError::LoginIncorrecto,
            AuthError::Unauthorized => ApiError::NoAutorizado,
            AuthError::NotFound => ApiError::NoEncontrado,
            AuthError::Conflict => ApiError::UsuarioYaExiste,
            AuthError::Forbidden => ApiError::Prohibido,
            AuthError::Database(e) => ApiError::ErrorDelServidor(e.to_string()),
            AuthError::PasswordHash(e) => ApiError::ErrorDelServidor(e.to_string()),
            AuthError::Token(e) => ApiError::ErrorDelServidor(e.to_string()),
        }
    }
}

#[derive(Serialize)]
struct CuerpoError {
    error: String,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.codigo_http()
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.codigo_http()).json(CuerpoError {
            error: self.mensaje(),
        })
    }
}
