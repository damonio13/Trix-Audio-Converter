//! Album Art Downloader
//!
//! Searches for and downloads album cover art from MusicBrainz and Discogs.
//! Supports embedding artwork into audio files via FFmpeg.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// A single album art result with source URL and dimensions.
pub struct AlbumArt {
    pub url: String,
    pub source: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 500;
const CACHE_TTL_DAYS: u64 = 30;

fn cache_key(source: &str, artist: &str, album: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update(b"|");
    hasher.update(artist.to_lowercase().as_bytes());
    hasher.update(b"|");
    hasher.update(album.to_lowercase().as_bytes());
    format!("{:x}.json", hasher.finalize())
}

fn get_cache_dir() -> PathBuf {
    crate::portable::Portable::data_dir().join("metadata_cache")
}

fn ensure_cache_dir() -> Result<(), String> {
    let dir = get_cache_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Falha ao criar cache dir: {}", e))
}

fn load_cached(source: &str, artist: &str, album: &str) -> Option<Vec<AlbumArt>> {
    let key = cache_key(source, artist, album);
    let path = get_cache_dir().join(key);
    
    if !path.exists() {
        return None;
    }
    
    let meta = std::fs::metadata(&path).ok()?;
    let modified = meta.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    
    if age > Duration::from_secs(CACHE_TTL_DAYS * 86400) {
        let _ = std::fs::remove_file(&path);
        return None;
    }
    
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_to_cache(source: &str, artist: &str, album: &str, arts: &[AlbumArt]) {
    if let Err(e) = ensure_cache_dir() {
        eprintln!("[AlbumArt] Cache dir error: {}", e);
        return;
    }
    
    let key = cache_key(source, artist, album);
    let path = get_cache_dir().join(key);
    
    if let Ok(json) = serde_json::to_string(arts) {
        let _ = std::fs::write(&path, json);
    }
}

async fn fetch_with_retry<F, Fut, T>(mut f: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_err = String::new();
    for attempt in 0..=MAX_RETRIES {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = e;
                if attempt < MAX_RETRIES {
                    let delay = BASE_RETRY_DELAY_MS * 2_u64.pow(attempt);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }
    if last_err.contains("connect") || last_err.contains("dns") || last_err.contains("timeout") {
        Err("Sem conexão com a internet para buscar capa de álbum. Usando cache local se disponível.".to_string())
    } else {
        Err(format!("Falha após {} tentativas: {}", MAX_RETRIES + 1, last_err))
    }
}

/// Downloads album cover art from MusicBrainz, Deezer, and iTunes.
pub struct AlbumArtDownloader {
    client: reqwest::Client,
}

impl AlbumArtDownloader {
    /// Creates a new AlbumArtDownloader with default configuration.
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            "AudioMasterPro/1.0 (contact@audiomaster.pro)".parse().unwrap(),
        );
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(5))
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    async fn fetch_json(&self, url: &str) -> Result<serde_json::Value, String> {
        fetch_with_retry(|| async {
            let resp = self.client.get(url).send().await
                .map_err(|e| format!("HTTP error: {}", e))?;
            resp.json().await.map_err(|e| format!("JSON parse error: {}", e))
        }).await
    }

    /// Searches MusicBrainz Cover Art Archive for album artwork.
    pub async fn search_musicbrainz(&self, artist: &str, album: &str) -> Result<Vec<AlbumArt>, String> {
        if let Some(cached) = load_cached("musicbrainz", artist, album) {
            return Ok(cached);
        }

        let query = format!("artist:{} album:{}", artist, album);
        let url = format!(
            "https://musicbrainz.org/ws/2/release-group/?query={}&fmt=json&limit=5",
            urlencoding::encode(&query)
        );

        let data = self.fetch_json(&url).await?;

        let mut arts = Vec::new();

        if let Some(groups) = data["release-groups"].as_array() {
            for group in groups {
                if let Some(id) = group["id"].as_str() {
                    let cover_url = format!(
                        "https://coverartarchive.org/release-group/{}/front-500",
                        id
                    );

                    let head = self.client.head(&cover_url)
                        .header("User-Agent", "AudioMasterPro/1.0")
                        .send()
                        .await;

                    if let Ok(resp) = head {
                        if resp.status().is_success() {
                            arts.push(AlbumArt {
                                url: cover_url,
                                source: "MusicBrainz".into(),
                                width: Some(500),
                                height: Some(500),
                            });
                        }
                    }
                }
            }
        }

        if !arts.is_empty() {
            save_to_cache("musicbrainz", artist, album, &arts);
        }

        Ok(arts)
    }

    /// Searches Deezer API for album artwork.
    pub async fn search_deezer(&self, artist: &str, album: &str) -> Result<Vec<AlbumArt>, String> {
        if let Some(cached) = load_cached("deezer", artist, album) {
            return Ok(cached);
        }

        let query = format!("{} {}", artist, album);
        let url = format!(
            "https://api.deezer.com/search/album?q={}",
            urlencoding::encode(&query)
        );

        let data = self.fetch_json(&url).await?;

        let mut arts = Vec::new();

        if let Some(data_arr) = data["data"].as_array() {
            for item in data_arr.iter().take(3) {
                if let Some(cover) = item["cover_xl"].as_str() {
                    arts.push(AlbumArt {
                        url: cover.to_string(),
                        source: "Deezer".into(),
                        width: Some(1000),
                        height: Some(1000),
                    });
                } else if let Some(cover) = item["cover_big"].as_str() {
                    arts.push(AlbumArt {
                        url: cover.to_string(),
                        source: "Deezer".into(),
                        width: Some(500),
                        height: Some(500),
                    });
                }
            }
        }

        if !arts.is_empty() {
            save_to_cache("deezer", artist, album, &arts);
        }

        Ok(arts)
    }

    /// Searches iTunes API for album artwork.
    pub async fn search_itunes(&self, artist: &str, album: &str) -> Result<Vec<AlbumArt>, String> {
        if let Some(cached) = load_cached("itunes", artist, album) {
            return Ok(cached);
        }

        let query = format!("{} {}", artist, album);
        let url = format!(
            "https://itunes.apple.com/search?entity=album&term={}&limit=5",
            urlencoding::encode(&query)
        );

        let data = self.fetch_json(&url).await?;

        let mut arts = Vec::new();

        if let Some(results) = data["results"].as_array() {
            for item in results {
                if let Some(art_url) = item["artworkUrl100"].as_str() {
                    let large = art_url.replace("100x100", "600x600");
                    arts.push(AlbumArt {
                        url: large,
                        source: "iTunes".into(),
                        width: Some(600),
                        height: Some(600),
                    });
                }
            }
        }

        if !arts.is_empty() {
            save_to_cache("itunes", artist, album, &arts);
        }

        Ok(arts)
    }

    /// Searches all available sources and returns the first result found.
    pub async fn search_all(&self, artist: &str, album: &str) -> Result<Vec<AlbumArt>, String> {
        let mut arts = Vec::new();

        let (mb, dz, it) = tokio::join!(
            self.search_musicbrainz(artist, album),
            self.search_deezer(artist, album),
            self.search_itunes(artist, album),
        );

        if let Ok(mut a) = mb { arts.append(&mut a); }
        if let Ok(mut a) = dz { arts.append(&mut a); }
        if let Ok(mut a) = it { arts.append(&mut a); }

        if arts.is_empty() {
            // All sources failed - likely offline
            return Err("Nenhuma capa encontrada. Verifique sua conexão com a internet.".into());
        }

        Ok(arts)
    }

    /// Downloads album art to a local file path.
    pub async fn download(&self, url: &str, dest: &str) -> Result<String, String> {
        if !crate::utils::is_safe_path(dest) {
            return Err("Invalid download destination".into());
        }

        let resp = self.client.get(url)
            .header("User-Agent", "AudioMasterPro/1.0")
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        let bytes = resp.bytes().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let dest_path = PathBuf::from(dest);
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }
        std::fs::write(&dest_path, &bytes)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(dest.to_string())
    }
}
