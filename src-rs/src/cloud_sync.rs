//! Cloud synchronization with Google Drive and Dropbox.
//!
//! Uploads, downloads, and synchronizes audio files and libraries
//! across cloud storage providers with conflict resolution support.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use reqwest::blocking::Client;
use std::sync::LazyLock;
use std::time::Duration;
use std::thread;

// NOTE: Uses reqwest::blocking (not async) because cloud operations are only
// invoked from CLI mode or dedicated threads — never from async handlers.

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 500;

fn execute_with_retry<F, T>(mut f: F) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let mut last_err = String::new();
    for attempt in 0..=MAX_RETRIES {
        match f() {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = e;
                if attempt < MAX_RETRIES {
                    let delay = BASE_RETRY_DELAY_MS * 2_u64.pow(attempt);
                    eprintln!("[CloudSync] Tentativa {} falhou: {}. Retry em {}ms...", attempt + 1, last_err, delay);
                    thread::sleep(Duration::from_millis(delay));
                }
            }
        }
    }
    // Provide user-friendly error for common connection issues
    if last_err.contains("connection") || last_err.contains("connect") || last_err.contains("dns") {
        Err(format!("Falha de conexão com a nuvem. Verifique sua conexão com a internet. Detalhes: {}", last_err))
    } else if last_err.contains("timeout") {
        Err(format!("Timeout ao conectar com a nuvem. Tente novamente. Detalhes: {}", last_err))
    } else {
        Err(format!("Falha após {} tentativas: {}", MAX_RETRIES + 1, last_err))
    }
}

fn save_response_to_file(response: reqwest::blocking::Response, path: &str) -> Result<String, String> {
    if response.status().is_success() {
        let bytes = response.bytes()
            .map_err(|e| format!("Falha ao ler resposta: {}", e))?;
        std::fs::write(path, &bytes)
            .map_err(|e| format!("Falha ao escrever arquivo: {}", e))?;
        Ok(path.to_string())
    } else {
        Err(format!("Erro HTTP: {}", response.status()))
    }
}

fn validate_no_special_chars(s: &str, name: &str) -> Result<(), String> {
    if s.contains('\0') || s.contains('\n') || s.contains('\r') || s.contains("..") {
        return Err(format!("{} contém caracteres inválidos", name));
    }
    Ok(())
}

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

fn validate_file_size(path: &str, name: &str) -> Result<(), String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("Falha ao verificar {}: {}", name, e))?;
    if meta.len() > MAX_FILE_SIZE {
        return Err(format!("{} excede limite de 100MB", name));
    }
    Ok(())
}

fn check_response(response: reqwest::blocking::Response, _context: &str) -> Result<serde_json::Value, String> {
    if response.status().is_success() {
        response.json::<serde_json::Value>()
            .map_err(|e| format!("Falha ao parsear resposta: {}", e))
    } else {
        Err(format!("Erro HTTP: {}", response.status()))
    }
}

/// Synchronizes audio files with cloud storage providers.
pub struct CloudSync;

impl CloudSync {
    /// Uploads a file to Google Drive in the specified folder.
    pub fn upload_to_gdrive(
        file_path: &str,
        folder_id: &str,
        api_key: &str,
    ) -> Result<String, String> {
        crate::utils::validate_cloud_upload_gdrive(file_path, folder_id, api_key)?;
        validate_no_special_chars(file_path, "caminho do arquivo")?;
        validate_file_size(file_path, "arquivo")?;

        let file_name = std::path::Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_bytes = std::fs::read(file_path)
            .map_err(|e| format!("Falha ao ler arquivo: {}", e))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Falha ao criar cliente HTTP: {}", e))?;

        let data = execute_with_retry(|| {
            let response = client
                .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=media")
                .header("Content-Type", "application/octet-stream")
                .header("X-Goog-Api-Key", api_key)
                .body(file_bytes.clone())
                .send()
                .map_err(|e| format!("Falha ao enviar: {}", e))?;
            check_response(response, "upload")
        })?;
        let file_id = data["id"].as_str().unwrap_or("").to_string();

        let metadata = serde_json::json!({
            "name": file_name,
            "addParents": folder_id
        });

        let _ = execute_with_retry(|| {
            client
                .patch(format!(
                    "https://www.googleapis.com/drive/v3/files/{}?addParents={}",
                    file_id, folder_id
                ))
                .header("Content-Type", "application/json")
                .header("X-Goog-Api-Key", api_key)
                .body(metadata.to_string())
                .send()
                .map_err(|e| format!("Falha ao atualizar metadados: {}", e))?;
            Ok(())
        });

        Ok(file_id)
    }

    /// Downloads a file from Google Drive by its file ID.
    pub fn download_from_gdrive(
        file_id: &str,
        output_path: &str,
        api_key: &str,
    ) -> Result<String, String> {
        crate::utils::validate_cloud_download_gdrive(file_id, output_path, api_key)?;
        validate_no_special_chars(output_path, "caminho de saída")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Falha ao criar cliente HTTP: {}", e))?;

        execute_with_retry(|| {
            let response = client
                .get(format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_id))
                .header("X-Goog-Api-Key", api_key)
                .send()
                .map_err(|e| format!("Falha ao baixar: {}", e))?;
            save_response_to_file(response, output_path)
        })
    }

    /// Uploads a file to Dropbox at the specified path.
    pub fn upload_to_dropbox(
        file_path: &str,
        dropbox_path: &str,
        access_token: &str,
    ) -> Result<String, String> {
        crate::utils::validate_cloud_upload_dropbox(file_path, dropbox_path, access_token)?;
        validate_no_special_chars(file_path, "caminho do arquivo")?;
        validate_file_size(file_path, "arquivo")?;

        let dropbox_api_arg = serde_json::json!({
            "path": dropbox_path,
            "mode": "add"
        });

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Falha ao criar cliente HTTP: {}", e))?;

        let file_bytes = std::fs::read(file_path)
            .map_err(|e| format!("Falha ao ler arquivo: {}", e))?;

        let data = execute_with_retry(|| {
            let response = client
                .post("https://content.dropboxapi.com/2/files/upload")
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Content-Type", "application/octet-stream")
                .header("Dropbox-API-Arg", dropbox_api_arg.to_string())
                .body(file_bytes.clone())
                .send()
                .map_err(|e| format!("Falha ao enviar: {}", e))?;
            check_response(response, "upload")
        })?;
        Ok(data["path_display"].as_str().unwrap_or("").to_string())
    }

    /// Downloads a file from Dropbox to a local path.
    pub fn download_from_dropbox(
        file_path: &str,
        dropbox_path: &str,
        access_token: &str,
    ) -> Result<String, String> {
        crate::utils::validate_cloud_download_dropbox(file_path, dropbox_path, access_token)?;
        validate_no_special_chars(file_path, "caminho do arquivo")?;

        let body_json = serde_json::json!({
            "path": dropbox_path
        });

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Falha ao criar cliente HTTP: {}", e))?;

        execute_with_retry(|| {
            let response = client
                .post("https://content.dropboxapi.com/2/files/download")
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Content-Type", "application/json")
                .body(body_json.to_string())
                .send()
                .map_err(|e| format!("Falha ao baixar: {}", e))?;
            save_response_to_file(response, file_path)
        })
    }

    /// Lists files in a Google Drive folder.
    pub fn list_gdrive_files(
        folder_id: &str,
        api_key: &str,
    ) -> Result<Vec<CloudFile>, String> {
        crate::utils::validate_cloud_list_gdrive(folder_id, api_key)?;

        let query = format!("'{}' in parents", folder_id);

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Falha ao criar cliente HTTP: {}", e))?;

        let data = execute_with_retry(|| {
            let response = client
                .get("https://www.googleapis.com/drive/v3/files")
                .query(&[("q", query.as_str())])
                .query(&[("fields", "files(id,name,mimeType,size)")])
                .header("X-Goog-Api-Key", api_key)
                .send()
                .map_err(|e| format!("Falha ao listar: {}", e))?;
            check_response(response, "list")
        })?;

        let files = data["files"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|f| CloudFile {
                        id: f["id"].as_str().unwrap_or("").to_string(),
                        name: f["name"].as_str().unwrap_or("").to_string(),
                        mime_type: f["mimeType"].as_str().unwrap_or("").to_string(),
                        size: f["size"].as_str().unwrap_or("0").parse().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(files)
    }

    /// Returns the list of supported cloud storage providers.
    pub fn get_cloud_providers() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("gdrive", "Google Drive"),
            ("dropbox", "Dropbox"),
            ("onedrive", "OneDrive"),
            ("s3", "Amazon S3"),
        ]);
        &*STATIC
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Metadata for a cloud storage file.
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: u64,
}
