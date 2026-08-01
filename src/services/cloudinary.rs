//! Cliente Cloudinary — Unsigned Uploads.

use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use crate::config::CloudinaryConfig;

#[derive(Debug, thiserror::Error)]
pub enum CloudinaryError {
    #[error("error de red al comunicarse con Cloudinary")]
    Http(#[from] reqwest::Error),

    #[error("Cloudinary respondió con error {status}: {body}")]
    Upload { status: u16, body: String },

    #[error("respuesta inválida de Cloudinary: falta `secure_url`")]
    MissingSecureUrl,
}

#[derive(Debug, Deserialize)]
struct CloudinaryUploadResponse {
    secure_url: Option<String>,
}

pub async fn subir_imagen(
    config: &CloudinaryConfig,
    bytes: Vec<u8>,
    filename: String,
    folder: Option<&str>,
) -> Result<String, CloudinaryError> {
    let endpoint = format!(
        "https://api.cloudinary.com/v1_1/{}/image/upload",
        config.cloud_name
    );

    let carpeta = folder.unwrap_or(&config.folder);
    let archivo = Part::bytes(bytes).file_name(filename);

    let formulario = Form::new()
        .part("file", archivo)
        .text("upload_preset", config.upload_preset.clone())
        .text("folder", carpeta.to_string());

    let respuesta = reqwest::Client::new()
        .post(&endpoint)
        .multipart(formulario)
        .send()
        .await?;

    let estado = respuesta.status();
    if !estado.is_success() {
        let cuerpo = respuesta.text().await.unwrap_or_default();
        return Err(CloudinaryError::Upload {
            status: estado.as_u16(),
            body: cuerpo,
        });
    }

    let datos: CloudinaryUploadResponse = respuesta.json().await?;
    datos.secure_url.ok_or(CloudinaryError::MissingSecureUrl)
}
