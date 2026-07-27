use std::future::{ready, Ready};

use actix_web::dev::Payload;
use actix_web::http::header;
use actix_web::{FromRequest, HttpRequest, web};

use crate::config::AppConfig;
use crate::error::ApiError;
use crate::services::auth;

pub struct AuthenticatedUser {
    pub user_id: i32,
}

pub struct OptionalAuthenticatedUser {
    pub user_id: Option<i32>,
}

impl FromRequest for AuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = (|| {
            let config = req
                .app_data::<web::Data<AppConfig>>()
                .map(|c| c.get_ref())
                .ok_or_else(|| ApiError::ErrorDelServidor("config no disponible".into()))?;

            let token = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .ok_or(ApiError::NoAutorizado)?;

            let user_id = auth::user_id_from_token(token, &config.jwt_secret)
                .map_err(|_| ApiError::NoAutorizado)?;

            Ok(AuthenticatedUser { user_id })
        })();

        ready(result)
    }
}

impl FromRequest for OptionalAuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = (|| {
            let config = req
                .app_data::<web::Data<AppConfig>>()
                .map(|c| c.get_ref())
                .ok_or_else(|| ApiError::ErrorDelServidor("config no disponible".into()))?;

            let user_id = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .and_then(|token| auth::user_id_from_token(token, &config.jwt_secret).ok());

            Ok(OptionalAuthenticatedUser { user_id })
        })();

        ready(result)
    }
}
