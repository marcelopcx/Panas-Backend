use bcrypt::BcryptError;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::usuario::{
    DescubrirItem, DescubrirQuery, ForgotPasswordRequest, LoginRequest, PasarDescubrirRequest,
    PerfilResponse, RegisterRequest, UpdateMeRequest, Usuario, UsuarioListItem, UsuarioListQuery,
    UsuarioPublicoResponse, display_name, normalizar_privacidad,
};

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

    #[error("solicitud inválida: {0}")]
    InvalidRequest(String),

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

#[derive(sqlx::FromRow)]
struct UsuarioPasswordRow {
    id_usuario: i32,
    username: String,
    email: String,
    url_avatar: Option<String>,
    password: String,
}

#[derive(sqlx::FromRow)]
struct PerfilRow {
    id_usuario: i32,
    username: String,
    email: String,
    url_avatar: Option<String>,
    fecha_registro: chrono::DateTime<Utc>,
    nombre: Option<String>,
    apellido: Option<String>,
    bio: Option<String>,
    privacidad: String,
}

fn optional_trim(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn resolver_avatar(body_url: Option<&String>, default_avatar_url: &str) -> String {
    body_url
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_avatar_url)
        .to_string()
}

fn conflicto_duplicado(err: sqlx::Error) -> AuthError {
    if let sqlx::Error::Database(db) = &err {
        if db.constraint().is_some() {
            return AuthError::Conflict;
        }
    }
    AuthError::Database(err)
}

fn username_desde_email(email: &str) -> String {
    let local = email.split('@').next().unwrap_or("user");
    let cleaned: String = local
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let base = if cleaned.is_empty() {
        "user".to_string()
    } else {
        cleaned.chars().take(40).collect()
    };
    format!("{base}_{}", Utc::now().timestamp() % 10_000)
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

pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    jwt_expiration_hours: i64,
    body: &LoginRequest,
) -> Result<(String, Usuario), AuthError> {
    let identificador = body
        .email
        .as_deref()
        .or(body.username.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(AuthError::InvalidCredentials)?;

    if body.password.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    let user = sqlx::query_as::<_, UsuarioPasswordRow>(
        r#"
        SELECT id_usuario, username, email, url_avatar, password
        FROM usuarios
        WHERE LOWER(username) = LOWER($1) OR LOWER(email) = LOWER($1)
        "#,
    )
    .bind(identificador)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::InvalidCredentials)?;

    if !bcrypt::verify(&body.password, &user.password)? {
        return Err(AuthError::InvalidCredentials);
    }

    let token = create_jwt(user.id_usuario, jwt_secret, jwt_expiration_hours)?;
    Ok((
        token,
        Usuario {
            id_usuario: user.id_usuario,
            username: user.username,
            email: user.email,
            url_avatar: user.url_avatar,
        },
    ))
}

pub async fn register(
    pool: &PgPool,
    body: &RegisterRequest,
    default_avatar_url: &str,
) -> Result<Usuario, AuthError> {
    let email = body.email.trim();
    if email.is_empty() {
        return Err(AuthError::InvalidRequest("email requerido".into()));
    }
    if body.password.len() < 8 {
        return Err(AuthError::InvalidRequest(
            "la contraseña debe tener al menos 8 caracteres".into(),
        ));
    }

    let nombre = body
        .full_name
        .as_deref()
        .or(body.nombre.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if nombre.map(|n| n.len()).unwrap_or(0) < 3 {
        return Err(AuthError::InvalidRequest(
            "el nombre completo es obligatorio (mínimo 3 caracteres)".into(),
        ));
    }

    let username = body
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| username_desde_email(email));

    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)?;
    let avatar = resolver_avatar(body.url_avatar.as_ref(), default_avatar_url);

    let user = sqlx::query_as::<_, Usuario>(
        r#"
        INSERT INTO usuarios (username, email, password, nombre, apellido, bio, url_avatar, privacidad)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'publico')
        RETURNING id_usuario, username, email, url_avatar
        "#,
    )
    .bind(&username)
    .bind(email)
    .bind(hash)
    .bind(nombre)
    .bind(optional_trim(&body.apellido))
    .bind(optional_trim(&body.bio))
    .bind(&avatar)
    .fetch_one(pool)
    .await
    .map_err(conflicto_duplicado)?;

    Ok(user)
}

/// Respuesta genérica (no revela si el email existe).
pub async fn forgot_password(
    pool: &PgPool,
    body: &ForgotPasswordRequest,
) -> Result<(), AuthError> {
    let email = body.email.trim();
    if email.is_empty() {
        return Err(AuthError::InvalidRequest("email requerido".into()));
    }

    // Placeholder: en producción enviarías un email con token de reset.
    let _existe: Option<i32> = sqlx::query_scalar(
        "SELECT id_usuario FROM usuarios WHERE LOWER(email) = LOWER($1)",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(())
}

pub async fn get_profile(pool: &PgPool, user_id: i32) -> Result<PerfilResponse, AuthError> {
    let row = sqlx::query_as::<_, PerfilRow>(
        r#"
        SELECT id_usuario, username, email, url_avatar, fecha_registro,
               nombre, apellido, bio, privacidad
        FROM usuarios
        WHERE id_usuario = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::NotFound)?;

    Ok(PerfilResponse {
        id_usuario: row.id_usuario,
        username: row.username.clone(),
        email: row.email,
        url_avatar: row.url_avatar,
        name: display_name(&row.nombre, &row.username),
        fecha_registro: row.fecha_registro,
        nombre: row.nombre,
        apellido: row.apellido,
        bio: row.bio,
        privacidad: row.privacidad,
    })
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: i32,
    body: &UpdateMeRequest,
) -> Result<PerfilResponse, AuthError> {
    let actual = get_profile(pool, user_id).await?;

    let username = body
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&actual.username);

    let email = body
        .email
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&actual.email);

    let password_hash = if let Some(password) = body.password.as_ref() {
        if password.len() < 8 {
            return Err(AuthError::InvalidRequest(
                "la contraseña debe tener al menos 8 caracteres".into(),
            ));
        }
        Some(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
    } else {
        None
    };

    let url_avatar = body
        .url_avatar
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or(actual.url_avatar.clone());

    let nombre = body
        .full_name
        .as_ref()
        .or(body.nombre.as_ref())
        .map(|s| optional_trim(&Some(s.clone())).map(|v| v.to_string()))
        .unwrap_or(actual.nombre.clone());

    let apellido = body
        .apellido
        .as_ref()
        .map(|s| optional_trim(&Some(s.clone())).map(|v| v.to_string()))
        .unwrap_or(actual.apellido.clone());

    let bio = body
        .bio
        .as_ref()
        .map(|s| optional_trim(&Some(s.clone())).map(|v| v.to_string()))
        .unwrap_or(actual.bio.clone());

    let privacidad = if let Some(raw) = body.privacidad.as_deref() {
        normalizar_privacidad(raw)
            .ok_or_else(|| {
                AuthError::InvalidRequest(
                    "privacidad debe ser publico, privado o solo_amigos".into(),
                )
            })?
            .to_string()
    } else {
        actual.privacidad.clone()
    };

    sqlx::query(
        r#"
        UPDATE usuarios
        SET username = $2,
            email = $3,
            password = COALESCE($4, password),
            url_avatar = $5,
            nombre = $6,
            apellido = $7,
            bio = $8,
            privacidad = $9
        WHERE id_usuario = $1
        "#,
    )
    .bind(user_id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(url_avatar)
    .bind(nombre)
    .bind(apellido)
    .bind(bio)
    .bind(privacidad)
    .execute(pool)
    .await
    .map_err(conflicto_duplicado)?;

    get_profile(pool, user_id).await
}

pub async fn actualizar_avatar(
    pool: &PgPool,
    user_id: i32,
    url_avatar: &str,
) -> Result<PerfilResponse, AuthError> {
    let result = sqlx::query(
        r#"
        UPDATE usuarios SET url_avatar = $2 WHERE id_usuario = $1
        "#,
    )
    .bind(user_id)
    .bind(url_avatar.trim())
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::NotFound);
    }

    get_profile(pool, user_id).await
}

pub async fn delete_account(pool: &PgPool, user_id: i32) -> Result<(), AuthError> {
    let result = sqlx::query("DELETE FROM usuarios WHERE id_usuario = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::NotFound);
    }

    Ok(())
}

pub async fn get_public_profile(
    pool: &PgPool,
    viewer_id: Option<i32>,
    user_id: i32,
) -> Result<UsuarioPublicoResponse, AuthError> {
    let row = sqlx::query_as::<_, PerfilRow>(
        r#"
        SELECT id_usuario, username, email, url_avatar, fecha_registro,
               nombre, apellido, bio, privacidad
        FROM usuarios
        WHERE id_usuario = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::NotFound)?;

    let puede_ver = match row.privacidad.as_str() {
        "publico" => true,
        "privado" => viewer_id == Some(user_id),
        "solo_amigos" => {
            if viewer_id == Some(user_id) {
                true
            } else if let Some(vid) = viewer_id {
                crate::services::amistad::son_amigos(pool, vid, user_id)
                    .await
                    .unwrap_or(false)
            } else {
                false
            }
        }
        _ => true,
    };

    if !puede_ver {
        return Err(AuthError::Forbidden);
    }

    Ok(UsuarioPublicoResponse {
        id_usuario: row.id_usuario,
        username: row.username.clone(),
        url_avatar: row.url_avatar,
        name: display_name(&row.nombre, &row.username),
        nombre: row.nombre,
        apellido: row.apellido,
        bio: row.bio,
        privacidad: row.privacidad,
    })
}

pub async fn listar_usuarios(
    pool: &PgPool,
    query: &UsuarioListQuery,
    exclude_user_id: Option<i32>,
) -> Result<Vec<UsuarioListItem>, AuthError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let q_pattern = query
        .q
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("%{}%", s.trim().to_lowercase()));

    #[derive(sqlx::FromRow)]
    struct Row {
        id_usuario: i32,
        username: String,
        email: String,
        url_avatar: Option<String>,
        fecha_registro: chrono::DateTime<Utc>,
        nombre: Option<String>,
        apellido: Option<String>,
        bio: Option<String>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT id_usuario, username, email, url_avatar, fecha_registro, nombre, apellido, bio
        FROM usuarios
        WHERE ($1::text IS NULL OR LOWER(username) LIKE $1 OR LOWER(email) LIKE $1 OR LOWER(COALESCE(nombre,'')) LIKE $1)
          AND ($2::int IS NULL OR id_usuario <> $2)
          AND privacidad = 'publico'
        ORDER BY LOWER(COALESCE(nombre, username)) ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&q_pattern)
    .bind(exclude_user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UsuarioListItem {
            id_usuario: r.id_usuario,
            username: r.username.clone(),
            email: r.email,
            url_avatar: r.url_avatar,
            name: display_name(&r.nombre, &r.username),
            fecha_registro: r.fecha_registro,
            nombre: r.nombre,
            apellido: r.apellido,
            bio: r.bio,
        })
        .collect())
}

/// Candidatos para Meet: públicos, no amigos, sin solicitud pendiente, no pasados.
pub async fn listar_descubrir(
    pool: &PgPool,
    id_usuario: i32,
    query: &DescubrirQuery,
) -> Result<Vec<DescubrirItem>, AuthError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 50);

    let rows = sqlx::query_as::<_, DescubrirItem>(
        r#"
        SELECT
            u.id_usuario,
            COALESCE(NULLIF(btrim(u.nombre), ''), u.username) AS name,
            u.url_avatar,
            u.bio,
            u.username
        FROM usuarios u
        WHERE u.id_usuario <> $1
          AND u.privacidad = 'publico'
          AND NOT EXISTS (
            SELECT 1 FROM solicitudes_amistad s
            WHERE s.estado IN ('pendiente', 'aceptada')
              AND (
                (s.id_remitente = $1 AND s.id_destinatario = u.id_usuario)
                OR (s.id_remitente = u.id_usuario AND s.id_destinatario = $1)
              )
          )
          AND NOT EXISTS (
            SELECT 1 FROM pases_descubrir p
            WHERE p.id_usuario = $1 AND p.id_pasado = u.id_usuario
          )
        ORDER BY RANDOM()
        LIMIT $2
        "#,
    )
    .bind(id_usuario)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn pasar_descubrir(
    pool: &PgPool,
    id_usuario: i32,
    body: &PasarDescubrirRequest,
) -> Result<(), AuthError> {
    if body.id_usuario == id_usuario {
        return Err(AuthError::InvalidRequest(
            "no puedes pasarte a ti mismo".into(),
        ));
    }

    let existe: Option<i32> = sqlx::query_scalar(
        "SELECT id_usuario FROM usuarios WHERE id_usuario = $1",
    )
    .bind(body.id_usuario)
    .fetch_optional(pool)
    .await?;

    if existe.is_none() {
        return Err(AuthError::NotFound);
    }

    sqlx::query(
        r#"
        INSERT INTO pases_descubrir (id_usuario, id_pasado)
        VALUES ($1, $2)
        ON CONFLICT (id_usuario, id_pasado) DO NOTHING
        "#,
    )
    .bind(id_usuario)
    .bind(body.id_usuario)
    .execute(pool)
    .await?;

    Ok(())
}
