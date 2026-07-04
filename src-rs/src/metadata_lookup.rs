//! Online metadata lookup via MusicBrainz and Discogs APIs.
//!
//! Queries music databases to retrieve artist, album, track, and
//! artwork information for audio files based on file metadata.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use sha2::{Digest, Sha256};

/// MusicBrainz release metadata from the MusicBrainz API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicBrainzRelease {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub date: String,
    pub country: String,
    pub track_count: u32,
    pub cover_url: Option<String>,
}

/// Discogs release metadata from the Discogs API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsRelease {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub genre: String,
    pub cover_url: Option<String>,
}

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 500;
const HTTP_TIMEOUT_SECS: u64 = 10;
const CONNECT_TIMEOUT_SECS: u64 = 5;
const CACHE_TTL_DAYS: u64 = 7;

fn cache_key(prefix: &str, query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(b"|");
    hasher.update(query.to_lowercase().as_bytes());
    format!("{:x}.json", hasher.finalize())
}

fn get_cache_dir() -> PathBuf {
    crate::portable::Portable::data_dir().join("metadata_cache")
}

fn ensure_cache_dir() -> Result<(), String> {
    let dir = get_cache_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Falha ao criar cache dir: {}", e))
}

fn load_cached_json<T: serde::de::DeserializeOwned>(prefix: &str, query: &str) -> Option<T> {
    let key = cache_key(prefix, query);
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

fn save_to_cache<T: serde::Serialize>(prefix: &str, query: &str, data: &T) {
    if let Err(e) = ensure_cache_dir() {
        eprintln!("[MetadataLookup] Cache dir error: {}", e);
        return;
    }

    let key = cache_key(prefix, query);
    let path = get_cache_dir().join(key);

    if let Ok(json) = serde_json::to_string(data) {
        let _ = std::fs::write(&path, json);
    }
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Falha ao criar cliente HTTP: {}", e))
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
    Err(format!("Falha após {} tentativas: {}", MAX_RETRIES + 1, last_err))
}

/// Online metadata lookup via MusicBrainz and Discogs APIs.
pub struct MetadataLookup;

impl MetadataLookup {
    /// Searches MusicBrainz for releases matching the query string.
    pub async fn search_musicbrainz(query: &str) -> Result<Vec<MusicBrainzRelease>, String> {
        if let Some(cached) = load_cached_json::<Vec<MusicBrainzRelease>>("musicbrainz", query) {
            return Ok(cached);
        }

        let client = build_client()?;
        let url = format!(
            "https://musicbrainz.org/ws/2/release/?query={}&fmt=json&limit=10",
            urlencoding::encode(query)
        );

        let releases = fetch_with_retry(|| {
            let client = client.clone();
            let url = url.clone();
            async move {
                let resp = client.get(&url)
                    .header("User-Agent", "TrixAudioConverter/1.0")
                    .send()
                    .await
                    .map_err(|e| {
                        if e.is_connect() || e.is_timeout() {
                            format!("Sem conexão com MusicBrainz: {}", e)
                        } else {
                            format!("Erro HTTP MusicBrainz: {}", e)
                        }
                    })?;

                resp.json::<serde_json::Value>()
                    .await
                    .map_err(|e| format!("Falha ao parsear resposta MusicBrainz: {}", e))
            }
        }).await?;

        let result: Vec<MusicBrainzRelease> = releases["releases"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|r| MusicBrainzRelease {
                        id: r["id"].as_str().unwrap_or("").to_string(),
                        title: r["title"].as_str().unwrap_or("").to_string(),
                        artist: r["artist-credit"]
                            .as_array()
                            .and_then(|a| a.first())
                            .and_then(|a| a["name"].as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        date: r["date"].as_str().unwrap_or("").to_string(),
                        country: r["country"].as_str().unwrap_or("").to_string(),
                        track_count: r["track-count"].as_u64().unwrap_or(0) as u32,
                        cover_url: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        save_to_cache("musicbrainz", query, &result);
        Ok(result)
    }

    /// Returns the Cover Art Archive URL for a MusicBrainz release.
    pub async fn get_musicbrainz_cover(release_id: &str) -> Result<String, String> {
        let url = format!(
            "https://coverartarchive.org/release/{}/front-500",
            release_id
        );
        Ok(url)
    }

    /// Searches Discogs for releases using an API token for authentication.
    pub async fn search_discogs(query: &str, token: &str) -> Result<Vec<DiscogsRelease>, String> {
        if let Some(cached) = load_cached_json::<Vec<DiscogsRelease>>("discogs", query) {
            return Ok(cached);
        }

        let client = build_client()?;
        let url = format!(
            "https://api.discogs.com/database/search?q={}&type=release&per_page=10",
            urlencoding::encode(query)
        );
        let auth_header = format!("Discogs token={}", token);

        let releases = fetch_with_retry(|| {
            let client = client.clone();
            let url = url.clone();
            let auth = auth_header.clone();
            async move {
                let resp = client.get(&url)
                    .header("Authorization", auth)
                    .send()
                    .await
                    .map_err(|e| {
                        if e.is_connect() || e.is_timeout() {
                            format!("Sem conexão com Discogs: {}", e)
                        } else {
                            format!("Erro HTTP Discogs: {}", e)
                        }
                    })?;

                resp.json::<serde_json::Value>()
                    .await
                    .map_err(|e| format!("Falha ao parsear resposta Discogs: {}", e))
            }
        }).await?;

        let result: Vec<DiscogsRelease> = releases["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|r| DiscogsRelease {
                        id: r["id"].as_u64().unwrap_or(0),
                        title: r["title"].as_str().unwrap_or("").to_string(),
                        artist: r["artist"]
                            .as_str()
                            .unwrap_or("Unknown")
                            .to_string(),
                        year: r["year"].as_u64().unwrap_or(0) as u32,
                        genre: r["genre"]
                            .as_array()
                            .and_then(|g| g.first())
                            .and_then(|g| g.as_str())
                            .unwrap_or("")
                            .to_string(),
                        cover_url: r["cover_image"].as_str().map(|s| s.to_string()),
                    })
                    .collect()
            })
            .unwrap_or_default();

        save_to_cache("discogs", query, &result);
        Ok(result)
    }

    /// Auto-tags a file by parsing its filename and enriching with MusicBrainz data.
    pub async fn auto_tag_from_filename(filename: &str) -> Result<crate::tag_editor::TagInfo, String> {
        let parsed = crate::auto_tagger::AutoTagger::parse_filename(filename);

        let mut tag = crate::tag_editor::TagInfo {
            artist: parsed.artist.unwrap_or_default(),
            title: parsed.title.unwrap_or_default(),
            track: parsed.track.unwrap_or(0).to_string(),
            year: parsed.year.unwrap_or_default(),
            genre: parsed.genre.unwrap_or_default(),
            ..Default::default()
        };

        let query = format!("{} {}", tag.artist, tag.title);
        if !tag.artist.is_empty() && !tag.title.is_empty() {
            match Self::search_musicbrainz(&query).await {
                Ok(releases) => {
                    if let Some(release) = releases.first() {
                        tag.album = release.title.clone();
                        if tag.year.is_empty() {
                            tag.year = release.date.clone();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[MetadataLookup] MusicBrainz lookup failed (offline?): {}", e);
                }
            }
        }

        Ok(tag)
    }
}
