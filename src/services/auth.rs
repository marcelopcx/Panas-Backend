use bcrypt::BcryptError;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("credenciales inválidas")]
    InvalidCredentials,

    #[error("no autorizado")]
    Unauthorized,

    #[error("usuario no encontrado")]
    NotFound,

    #[error("conflicto: usuario o correo ya registrado")]
    Conflict,

    #[error("no autorizado")]
    Forbidden,

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),

    #[error("error al verificar contraseña")]
    PasswordHash(#[from] BcryptError),

    #[error("error al generar token")]
    Token(#[from] jsonwebtoken::errors::Error),
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
}

pub fn user_id_from_token(token: &str, secret: &str) -> Result<i32, AuthError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AuthError::Unauthorized)?;

    Ok(data.claims.sub)
}

pub fn create_jwt(
    user_id: i32,
    secret: &str,
    expiration_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (Utc::now() + Duration::hours(expiration_hours)).timestamp() as usize;
    let claims = Claims { sub: user_id, exp };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
