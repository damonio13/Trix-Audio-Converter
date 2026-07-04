//! Auto-Updater
//!
//! Checks for new versions on GitHub Releases and handles download/install.
//! Supports silent and interactive update modes.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

/// Information about an available software update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current: String,
    pub latest: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
}

/// Checks GitHub Releases for new versions and reports update availability.
pub struct AutoUpdater {
    current_version: String,
}

impl AutoUpdater {
    /// Creates a new AppUpdater with the GitHub releases endpoint pre-configured.
    pub fn new() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Checks GitHub Releases for a newer version and returns an UpdateInfo struct.
    pub async fn check_for_updates(&self) -> UpdateInfo {
        let url = "https://api.github.com/repos/trix-audio-converter/trix-audio-converter/releases/latest";

        match reqwest::Client::new()
            .get(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "AudioMasterPro")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    let tag = data.get("tag_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim_start_matches('v')
                        .to_string();

                    let notes = data.get("body")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let download_url = data.get("assets")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| {
                            arr.iter().find_map(|asset| {
                                let name = asset.get("name")?.as_str()?.to_lowercase();
                                if name.contains("windows") || name.ends_with(".exe") || name.ends_with(".zip") {
                                    asset.get("browser_download_url").and_then(|v| v.as_str()).map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                        })
                        .or_else(|| data.get("html_url").and_then(|v| v.as_str()).map(|s| s.to_string()));

                    let available = self.is_newer(&tag);

                    return UpdateInfo {
                        available,
                        current: self.current_version.clone(),
                        latest: Some(tag),
                        notes,
                        url: download_url,
                        error: None,
                    };
                }

                UpdateInfo {
                    available: false,
                    current: self.current_version.clone(),
                    latest: None,
                    notes: None,
                    url: None,
                    error: Some("Falha ao parsear resposta".into()),
                }
            }
            Err(e) => UpdateInfo {
                available: false,
                current: self.current_version.clone(),
                latest: None,
                notes: None,
                url: None,
                error: Some(if e.is_connect() || e.is_timeout() {
                    "Sem conexão com a internet para verificar atualizações".to_string()
                } else {
                    format!("Falha ao verificar atualizacoes: {}", e)
                }),
            },
        }
    }

    fn is_newer(&self, remote_version: &str) -> bool {
        let remote_parts: Vec<u32> = remote_version.split('.')
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect();
        let local_parts: Vec<u32> = self.current_version.split('.')
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect();
        remote_parts > local_parts
    }
}
