use actix_web::{delete, get, patch, post, web, HttpResponse};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::config::AppConfig;
use crate::error::ApiError;
use crate::models::usuario::{LoginRequest, RegisterRequest, UpdateMeRequest, UsuarioListQuery};
use crate::services::auth;

#[post("/auth/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = auth::register(pool.get_ref(), &body, &config.default_avatar_url).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({ "user": user })))
}

#[post("/auth/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let (token, user) = auth::login(
        pool.get_ref(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
        &body,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token, "user": user })))
}

#[get("/auth/me")]
pub async fn get_me(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let perfil = auth::get_profile(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::Ok().json(perfil))
}

#[patch("/auth/me")]
pub async fn patch_me(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    body: web::Json<UpdateMeRequest>,
) -> Result<HttpResponse, ApiError> {
    let perfil = auth::update_profile(pool.get_ref(), user.user_id, &body).await?;
    Ok(HttpResponse::Ok().json(perfil))
}

#[delete("/auth/me")]
pub async fn delete_me(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    auth::delete_account(pool.get_ref(), user.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/usuarios")]
pub async fn listar_usuarios(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    query: web::Query<UsuarioListQuery>,
) -> Result<HttpResponse, ApiError> {
    let usuarios =
        auth::listar_usuarios(pool.get_ref(), &query, Some(user.user_id)).await?;
    Ok(HttpResponse::Ok().json(usuarios))
}

#[get("/usuarios/{id}")]
pub async fn get_usuario(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let perfil = auth::get_public_profile(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(perfil))
}
