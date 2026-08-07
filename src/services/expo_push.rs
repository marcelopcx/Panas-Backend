//! Cliente Expo Push Notifications.

use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;

const EXPO_PUSH_URL: &str = "https://exp.host/--/api/v2/push/send";

#[derive(Debug, Serialize)]
struct ExpoPushMessage {
    to: String,
    title: String,
    body: String,
    sound: &'static str,
    data: Value,
    #[serde(rename = "channelId")]
    channel_id: &'static str,
}

fn title_for_tipo(tipo: &str) -> &'static str {
    match tipo {
        "mensaje" => "Nuevo mensaje",
        "solicitud_amistad" => "Solicitud de amistad",
        "solicitud_aceptada" => "Amistad aceptada",
        _ => "Panas",
    }
}

pub async fn obtener_token(pool: &PgPool, id_usuario: i32) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT expo_push_token FROM push_tokens WHERE id_usuario = $1
        "#,
    )
    .bind(id_usuario)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn guardar_token(
    pool: &PgPool,
    id_usuario: i32,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO push_tokens (id_usuario, expo_push_token, actualizado_en)
        VALUES ($1, $2, CURRENT_TIMESTAMP)
        ON CONFLICT (id_usuario) DO UPDATE
        SET expo_push_token = EXCLUDED.expo_push_token,
            actualizado_en = CURRENT_TIMESTAMP
        "#,
    )
    .bind(id_usuario)
    .bind(token.trim())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn eliminar_token(pool: &PgPool, id_usuario: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM push_tokens WHERE id_usuario = $1")
        .bind(id_usuario)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn enviar_push(
    token: &str,
    tipo: &str,
    mensaje: &str,
    id_referencia: Option<i32>,
) {
    let payload = ExpoPushMessage {
        to: token.to_string(),
        title: title_for_tipo(tipo).to_string(),
        body: mensaje.to_string(),
        sound: "default",
        data: serde_json::json!({
            "tipo": tipo,
            "id_referencia": id_referencia,
        }),
        channel_id: "default",
    };

    match reqwest::Client::new()
        .post(EXPO_PUSH_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("Expo Push error {status}: {body}");
            }
        }
        Err(err) => {
            eprintln!("Expo Push network error: {err}");
        }
    }
}

/// Envía push al usuario si tiene token registrado. Nunca propaga errores.
pub async fn notificar_usuario(
    pool: &PgPool,
    id_usuario: i32,
    tipo: &str,
    mensaje: &str,
    id_referencia: Option<i32>,
) {
    let Some(token) = obtener_token(pool, id_usuario).await else {
        return;
    };
    enviar_push(&token, tipo, mensaje, id_referencia).await;
}
