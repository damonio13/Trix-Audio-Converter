//! HTTP API Server (axum)
//!
//! REST API endpoints for audio conversion, file scanning, format listing,
//! cloud sync, and window management. Uses axum with tower-http middleware
//! for CORS, body limits, and static file serving.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;

use crate::converter::AudioConverter;
use crate::formats::{OUTPUT_FORMATS, SAMPLE_RATES, CHANNELS};
use crate::logger::ConversionLogger;
use crate::plugins::PluginManager;
use crate::preview::AudioPreview;
use crate::queue_manager::QueueManager;
use crate::scheduler::ConversionScheduler;
use crate::scanner::FileScanner;
use std::hash::{Hash, Hasher};
use subtle::ConstantTimeEq;
use crate::video_extractor::VideoExtractor;

// ── Typed request/response structs ──

/// Request body for the `/api/scan` endpoint.
/// Accepts either individual file `paths` or `folders` to scan recursively.
#[derive(Deserialize)]
struct ScanFoldersRequest {
    /// Individual file or directory paths dropped by the user.
    #[serde(default)]
    paths: Vec<String>,
    /// Folder paths to scan recursively (used by the folder picker flow).
    #[serde(default)]
    folders: Vec<String>,
    /// Target format key used to pre-compute output filenames during scan.
    #[serde(default)]
    format_key: String,
}

/// Accepts `codec_copy` as either a boolean (`true`/`false`) or a string
/// (`"auto"` / `"never"`) for flexibility between callers.
#[derive(Deserialize)]
#[serde(untagged)]
enum CodecCopyValue {
    Bool(bool),
    String(String),
}

/// Full request body for the `/api/start` endpoint.
/// Fields accept both snake_case and camelCase aliases for compatibility
/// with the TypeScript frontend and future CLI callers.
#[derive(Deserialize)]
struct StartConversionRequest {
    #[serde(default)]
    sample_rate: String,
    #[serde(alias = "sampleRate", default)]
    sample_rate_alt: Option<String>,
    #[serde(default)]
    channels: String,
    #[serde(default)]
    volume: i32,
    #[serde(default)]
    trim_start: f64,
    #[serde(alias = "trimStart", default)]
    trim_start_alt: Option<f64>,
    #[serde(default)]
    trim_end: f64,
    #[serde(alias = "trimEnd", default)]
    trim_end_alt: Option<f64>,
    #[serde(default)]
    output_pattern: String,
    #[serde(default)]
    codec_copy: Option<String>,
    #[serde(alias = "codecCopy", default)]
    codec_copy_alt: Option<CodecCopyValue>,
    #[serde(default)]
    output_subfolder: bool,
    #[serde(default)]
    max_output_size_mb: f64,
    #[serde(default)]
    output_directory: String,
    #[serde(alias = "outputDirectory", default)]
    output_directory_alt: Option<String>,
    #[serde(alias = "outputInSameFolder", default = "default_output_in_same_folder")]
    output_in_same_folder: bool,
    #[serde(default)]
    output_suffix: String,
    #[serde(alias = "outputSuffix", default)]
    output_suffix_alt: Option<String>,
    #[serde(default)]
    format: String,
    #[serde(alias = "bitRate", default)]
    bit_rate: Option<String>,
    #[serde(alias = "format_key", default)]
    format_key_alt: Option<String>,
    #[serde(default)]
    formats: Option<Vec<String>>,
    #[serde(default)]
    files: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    folders: Vec<String>,
}

/// Serde default for `output_in_same_folder` — saves to the source file's directory.
fn default_output_in_same_folder() -> bool {
    true
}

impl StartConversionRequest {
    /// Returns the canonical sample-rate key, accepting both the key and the
    /// raw Hz value (e.g. `"44100"` → `"44100 Hz"`) for robustness.
    fn effective_sample_rate(&self) -> &str {
        let raw: &str = if !self.sample_rate.is_empty() {
            &self.sample_rate
        } else {
            self.sample_rate_alt.as_deref().unwrap_or("Original")
        };
        if SAMPLE_RATES.contains_key(raw) {
            raw
        } else {
            SAMPLE_RATES.iter()
                .find(|(_, v)| **v == raw)
                .map(|(k, _)| *k)
                .unwrap_or("Original")
        }
    }

    /// Returns the canonical channel-mode key (e.g. `"Stereo"`, `"Mono"`).
    fn effective_channels(&self) -> &str {
        let raw: &str = &self.channels;
        if CHANNELS.contains_key(raw) {
            raw
        } else {
            CHANNELS.iter()
                .find(|(_, v)| **v == raw)
                .map(|(k, _)| *k)
                .unwrap_or("Original")
        }
    }

    /// Returns the trim start in seconds, preferring the camelCase alias field.
    fn effective_trim_start(&self) -> f64 {
        self.trim_start_alt.unwrap_or(self.trim_start)
    }

    /// Returns the trim end in seconds, preferring the camelCase alias field.
    fn effective_trim_end(&self) -> f64 {
        self.trim_end_alt.unwrap_or(self.trim_end)
    }

    /// Returns the effective output directory, preferring the camelCase alias.
    fn effective_output_directory(&self) -> &str {
        self.output_directory_alt.as_deref().unwrap_or(&self.output_directory)
    }

    /// Returns the output filename suffix (e.g. `"_trix"`), preferring the camelCase alias.
    fn effective_output_suffix(&self) -> &str {
        self.output_suffix_alt.as_deref().unwrap_or(&self.output_suffix)
    }

    /// Resolves codec-copy mode from either `codec_copy` (string) or
    /// `codec_copy_alt` (bool/string). Returns `"auto"`, `"never"`, or a codec name.
    fn effective_codec_copy(&self) -> &str {
        if let Some(ref s) = self.codec_copy {
            return s;
        }
        match &self.codec_copy_alt {
            Some(CodecCopyValue::Bool(true)) => "auto",
            Some(CodecCopyValue::String(s)) => s,
            _ => "never",
        }
    }

    /// Returns the target format key, preferring `format_key_alt` over `format`.
    fn effective_format_key(&self) -> &str {
        self.format_key_alt.as_deref().unwrap_or(&self.format)
    }

    /// Validates user-supplied path and suffix fields against path-traversal
    /// and forbidden-character rules before passing them to the converter.
    fn validate(&self) -> Result<(), String> {
        if crate::utils::has_invalid_chars(self.effective_output_suffix()) {
            return Err("Sufixo de saida invalido".into());
        }
        if crate::utils::has_invalid_chars(&self.output_pattern) {
            return Err("Padrao de saida invalido".into());
        }
        let out_dir = self.effective_output_directory();
        if !self.output_in_same_folder && !out_dir.is_empty() && !is_valid_path(out_dir) {
            return Err("Caminho de saida invalido".into());
        }
        Ok(())
    }
}

/// Generic single-path request. `mode` is used by `/open-folder` to
/// distinguish between opening a folder in Explorer and picking a new one.
#[derive(Deserialize)]
struct PathRequest {
    /// Absolute filesystem path.
    #[serde(default)]
    path: String,
    /// Operation mode — e.g. `"select"` to open a folder-picker dialog.
    #[serde(default)]
    mode: String,
}

/// Request body for endpoints that operate on a single audio file path.
#[derive(Deserialize)]
struct AudioPathRequest {
    /// Absolute path to the audio file.
    path: String,
}

/// Request body for `/api/video/extract` — extract audio from a video file.
#[derive(Deserialize)]
struct ExtractAudioRequest {
    /// Absolute path to the source video file.
    input: String,
    /// Absolute path for the output audio file.
    output: String,
    /// Target audio format (default: `"mp3"`).
    #[serde(default = "default_format")]
    format: String,
    /// Optional bitrate override (e.g. `"192k"`).
    bitrate: Option<String>,
}

/// Default audio format for video extraction when not specified by the caller.
fn default_format() -> String {
    "mp3".to_string()
}

/// Request body for `/api/clear-cache`.
/// Two-phase: first call returns a file count; second call with `confirm: true` deletes them.
#[derive(Deserialize)]
struct ClearCacheRequest {
    /// When `true`, actually deletes cached files. When `false`, returns a dry-run count.
    #[serde(default)]
    confirm: bool,
}

/// A single file entry returned by the `/api/scan` endpoint.
#[derive(Debug, Clone, Serialize)]
struct FileEntry {
    /// Deterministic hex ID derived from the file path (used as React key).
    id: String,
    /// File name without directory.
    name: String,
    /// Absolute path to the file.
    path: String,
    /// Lowercase file extension without the leading dot.
    ext: String,
    /// File size in bytes.
    size: u64,
    /// Initial status — always `"pending"` after a scan.
    status: String,
    /// Initial progress percentage — always `0` after a scan.
    progress: u8,
}

/// A single user action recorded in the conversion history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    timestamp: String,
    action: String,
    details: String,
}

const MAX_AUTH_FAILURES: u32 = 10;
const AUTH_LOCKOUT_DURATION: Duration = Duration::from_secs(300);

/// Per-IP rate limiter for authentication attempts with lockout.
pub struct RateLimiter {
    attempts: HashMap<std::net::IpAddr, (u32, Instant)>,
}

impl RateLimiter {
    fn new() -> Self {
        RateLimiter {
            attempts: HashMap::new(),
        }
    }

    fn check_and_record(&mut self, ip: std::net::IpAddr) -> bool {
        let now = Instant::now();
        let entry = self.attempts.entry(ip).or_insert((0, now));

        if now.duration_since(entry.1) > AUTH_LOCKOUT_DURATION {
            *entry = (1, now);
            return true;
        }

        if entry.0 >= MAX_AUTH_FAILURES {
            return false;
        }

        entry.0 += 1;
        entry.1 = now;
        true
    }

    fn is_locked(&self, ip: &std::net::IpAddr) -> bool {
        if let Some((count, since)) = self.attempts.get(ip) {
            if Instant::now().duration_since(*since) > AUTH_LOCKOUT_DURATION {
                return false;
            }
            *count >= MAX_AUTH_FAILURES
        } else {
            false
        }
    }
}

/// Shared application state for the axum HTTP server.
pub struct AppState {
    pub converter: RwLock<AudioConverter>,
    pub logger: RwLock<ConversionLogger>,
    pub scheduler: RwLock<ConversionScheduler>,
    pub queue_manager: QueueManager,
    pub plugin_manager: RwLock<PluginManager>,
    pub history: RwLock<Vec<HistoryEntry>>,
    pub stats_path: PathBuf,
    pub undo_path: PathBuf,
    pub profiles_path: PathBuf,
    pub presets_path: PathBuf,
    pub api_token: String,
    pub rate_limiter: RwLock<RateLimiter>,
}

/// Generates a cryptographically random 32-character hex API token using UUIDv4.
fn generate_token() -> String {
    let uuid = uuid::Uuid::new_v4();
    uuid.to_string().replace('-', "")
}

/// Compares two strings in constant time to prevent timing-based token enumeration.
fn constant_time_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Returns `true` if the file path has a known audio extension from [`INPUT_EXTENSIONS`].
/// Uses a stack-allocated buffer to avoid heap allocation during extension comparison.
fn has_audio_extension(path: &str) -> bool {
    if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos..];
        let mut buf = [0u8; 256];
        let bytes = ext.as_bytes();
        let len = bytes.len().min(buf.len());
        for i in 0..len {
            buf[i] = bytes[i].to_ascii_lowercase();
        }
        if let Ok(ext_str) = std::str::from_utf8(&buf[..len]) {
            return crate::formats::INPUT_EXTENSIONS.contains(ext_str);
        }
    }
    false
}

/// Validates a path against path-traversal patterns and safety rules.
/// Returns a ready-made JSON error response on failure, usable as an early-return in handlers.
fn validate_path(path: &str) -> Result<(), Json<serde_json::Value>> {
    if path.is_empty() || crate::utils::path_has_traversal(path) || !crate::utils::is_safe_path(path) {
        return Err(Json(serde_json::json!({"success": false, "error": "Caminho invalido"})));
    }
    Ok(())
}

/// Returns `true` if the path passes both traversal and safety checks.
/// Lightweight version of [`validate_path`] that returns a `bool` instead of a JSON error.
fn is_valid_path(path: &str) -> bool {
    !crate::utils::path_has_traversal(path) && crate::utils::is_safe_path(path)
}

/// Extracts and validates an audio file path from a polymorphic JSON body.
/// Accepts either `{"0": "/path"}` (positional) or a bare string value.
fn extract_audio_path(body: &serde_json::Value) -> Result<String, StatusCode> {
    let path = body.get("0").and_then(|v| v.as_str())
        .or_else(|| body.as_str())
        .unwrap_or("");
    if !is_valid_path(path) || !has_audio_extension(path) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(path.to_string())
}

/// Returns a [`PathBuf`] for a named file inside the app data directory.
fn data_file(name: &str) -> PathBuf {
    crate::portable::Portable::data_dir().join(name)
}

/// Builds a [`FileEntry`] from a filesystem path, generating a deterministic ID
/// by hashing `id_source` (typically the absolute path string).
fn build_file_entry(path: &std::path::Path, id_source: &str) -> FileEntry {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id_source.hash(&mut hasher);
    let id = format!("{:016x}", hasher.finish());
    let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    FileEntry {
        id, name, path: path.to_string_lossy().to_string(),
        ext, size, status: "pending".into(), progress: 0,
    }
}

/// Returns `true` if the path has a known audio or video extension.
/// Checks against [`INPUT_EXTENSIONS`] first, then a static video extension list.
fn path_has_audio_or_video_extension(path: &str) -> bool {
    let ext = match path.rfind('.') {
        Some(pos) => &path[pos..],
        None => return false,
    };
    let mut buf = [0u8; 256];
    let bytes = ext.as_bytes();
    let len = bytes.len().min(buf.len());
    for i in 0..len {
        buf[i] = bytes[i].to_ascii_lowercase();
    }
    if let Ok(ext_lower) = std::str::from_utf8(&buf[..len]) {
        if crate::formats::INPUT_EXTENSIONS.contains(ext_lower) {
            return true;
        }
        static VIDEO_EXTS: &[&str] = &[
            ".mkv", ".avi", ".mov", ".m4v", ".flv", ".3gp", ".mpg", ".mpeg",
            ".ts", ".vob", ".ogv", ".f4v", ".rm", ".rmvb", ".asf", ".divx",
            ".m2ts", ".mts", ".nsv", ".mp4", ".wmv", ".webm", ".mka",
        ];
        return VIDEO_EXTS.iter().any(|v| *v == ext_lower);
    }
    false
}

/// Bearer-token authentication middleware with per-IP rate limiting.
///
/// Loopback requests (`127.0.0.1`) bypass token checks because the WebView and
/// CLI both run on the same machine. External requests must provide a valid
/// `Authorization: Bearer <token>` header or receive `401 Unauthorized`.
/// After [`MAX_AUTH_FAILURES`] consecutive failures the IP is locked out for
/// [`AUTH_LOCKOUT_DURATION`] seconds.
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
        .or_else(|| {
            request
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip())
        })
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

    if client_ip.is_loopback() {
        return Ok(next.run(request).await);
    }

    let auth_header = request.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            if constant_time_eq(token, &state.api_token) {
                Ok(next.run(request).await)
            } else {
                if !client_ip.is_loopback() {
                    let mut limiter = state.rate_limiter.write().await;
                    limiter.check_and_record(client_ip);
                }
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            if !client_ip.is_loopback() {
                let mut limiter = state.rate_limiter.write().await;
                limiter.check_and_record(client_ip);
            }
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Content-Security-Policy middleware.
///
/// Injects `Content-Security-Policy`, `X-Content-Type-Options`, and
/// `X-Frame-Options` headers on every response to harden the WebView context.
async fn csp_middleware(request: Request, next: middleware::Next) -> Response {
    let mut response = next.run(request).await;

    let csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; img-src 'self' data:; connect-src 'self'; font-src 'self' https://fonts.gstatic.com; object-src 'none'; frame-src 'none'; base-uri 'self'; form-action 'self';";

    response.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_str(csp).unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
    );

    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );

    response.headers_mut().insert(
        "x-frame-options",
        HeaderValue::from_static("DENY"),
    );

    response
}

/// Starts the HTTP API server on an auto-assigned port.
pub async fn serve() {
    serve_with_port(None).await;
}

/// Starts the HTTP API server, optionally with a pre-created AudioConverter.
pub async fn serve_with_port(converter: Option<Arc<AudioConverter>>) {
    let stats_path = data_file("stats.json");
    let undo_path = data_file("undo_history.json");
    let profiles_path = data_file("profiles.json");
    let presets_path = data_file("presets.json");

    let api_token = generate_token();

    let converter = converter.unwrap_or_else(|| Arc::new(AudioConverter::new(0)));

    let state = Arc::new(AppState {
        converter: RwLock::new((*converter).clone()),
        logger: RwLock::new(ConversionLogger::new()),
        scheduler: RwLock::new(ConversionScheduler::new()),
        queue_manager: QueueManager::new(),
        plugin_manager: RwLock::new(PluginManager::new()),
        history: RwLock::new(Vec::new()),
        stats_path,
        undo_path,
        profiles_path,
        presets_path,
        api_token: api_token.clone(),
        rate_limiter: RwLock::new(RateLimiter::new()),
    });

    let frontend_dir = {
        let candidate = std::env::current_dir().unwrap_or_default().join("..").join("dist");
        if candidate.join("index.html").exists() {
            candidate
        } else {
            let candidate2 = std::env::current_dir().unwrap_or_default().join("dist");
            if candidate2.join("index.html").exists() {
                candidate2
            } else {
                std::env::current_exe()
                    .unwrap_or_default()
                    .parent().unwrap_or(std::path::Path::new("."))
                    .join("..").join("..").join("..").join("..").join("dist")
            }
        }
    };

    let api_routes = Router::new()
        // Core
        .route("/formats", get(get_formats))
        .route("/sample-rates", get(get_sample_rates))
        .route("/channels", get(get_channels))
        .route("/scan", post(scan_folders))
        .route("/start", post(start_conversion))
        .route("/cancel", post(cancel_conversion))
        .route("/status", get(get_status))
        .route("/status/:jobId", get(get_status))
        .route("/waveform", post(get_waveform))
        .route("/spectrum", post(get_spectrum))
        .route("/open-folder", post(open_folder))
        .route("/clear-cache", post(clear_cache))
        .route("/context-menu/register", post(register_context_menu))
        .route("/context-menu/unregister", post(unregister_context_menu))
        // File picker
        .route("/pick-files", post(pick_files))
        .route("/dropped-files", get(get_dropped_files))
        // Video
        .route("/video/probe", post(probe_video))
        .route("/video/extract", post(extract_audio_from_video))
        // Window
        .route("/window/close", post(window_close))
        .route("/window/minimize", post(window_minimize))
        .route("/window/maximize", post(window_maximize))
        // Crash logs
        .route("/crash-logs", get(get_crash_logs))
        .route("/crash-logs/read", post(read_crash_log))
        .route("/crash-logs/delete", post(delete_crash_log))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Health check endpoint (no auth required)
    let health_route = Router::new()
        .route("/health", get(health_check));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/api", health_route)
        .fallback_service(ServeDir::new(&frontend_dir))
        .layer(middleware::from_fn(csp_middleware))
        .layer(ConcurrencyLimitLayer::new(32))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .with_state(state.clone());

    // Use fixed port 3939 so the Vite dev proxy can reach us reliably.
    // Fall back to a random port if 3939 is already taken.
    let addr_fixed = SocketAddr::from(([127, 0, 0, 1], 3939));
    let addr_random = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = match tokio::net::TcpListener::bind(addr_fixed).await {
        Ok(l) => l,
        Err(_) => tokio::net::TcpListener::bind(addr_random).await
            .unwrap_or_else(|e| panic!("Falha ao iniciar servidor: {}", e)),
    };
    let port = listener.local_addr()
        .unwrap_or_else(|e| panic!("Falha ao obter endereco: {}", e))
        .port();

    let mut allowed_origins = vec![
        format!("http://localhost:{}", port).parse().unwrap_or_else(|_| HeaderValue::from_static("http://localhost:0")),
        format!("http://127.0.0.1:{}", port).parse().unwrap_or_else(|_| HeaderValue::from_static("http://127.0.0.1:0")),
    ];
    if let Ok(dev_origin) = "http://localhost:8888".parse::<HeaderValue>() {
        allowed_origins.push(dev_origin);
    }
    if let Ok(dev_origin_127) = "http://127.0.0.1:8888".parse::<HeaderValue>() {
        allowed_origins.push(dev_origin_127);
    }

    let app = app.layer(
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    );

    let _ = crate::SERVER_PORT.set(port);
    let _ = crate::API_TOKEN.set(api_token);

    println!("Trix Audio Converter - http://localhost:{}", port);

    axum::serve(listener, app).await
        .unwrap_or_else(|e| panic!("Servidor encerrou com erro: {}", e));
}

// ── Handlers ──

/// `GET /api/health` — simple liveness probe (no auth required).
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// `GET /api/formats` — returns all 106 supported output formats.
/// Response is cached in a `OnceLock` after the first call.
async fn get_formats() -> Json<serde_json::Value> {
    static CACHE: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    Json(CACHE.get_or_init(|| serde_json::to_value(&*OUTPUT_FORMATS).unwrap_or_default()).clone())
}

/// `GET /api/sample-rates` — returns the map of human-readable sample-rate labels to Hz values.
async fn get_sample_rates() -> Json<serde_json::Value> {
    static CACHE: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    Json(CACHE.get_or_init(|| serde_json::to_value(&*SAMPLE_RATES).unwrap_or_default()).clone())
}

/// `GET /api/channels` — returns the map of channel-mode labels to FFmpeg values.
async fn get_channels() -> Json<serde_json::Value> {
    static CACHE: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    Json(CACHE.get_or_init(|| serde_json::to_value(&*CHANNELS).unwrap_or_default()).clone())
}

/// Recursively walks `dir` and appends all supported audio/video files to `files`.
fn scan_dir_recursive(dir: &std::path::Path, files: &mut Vec<serde_json::Value>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(&path, files);
            } else if path.is_file() {
                let path_str = path.to_string_lossy();
                if path_has_audio_or_video_extension(&path_str) {
                    let entry_val = build_file_entry(&path, &path_str);
                    files.push(serde_json::to_value(entry_val).unwrap_or_default());
                }
            }
        }
    }
}

/// `POST /api/scan` — scans paths or folders for supported audio/video files.
/// Returns a JSON array of [`FileEntry`] objects ready to populate the queue.
async fn scan_folders(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<ScanFoldersRequest>,
) -> Json<serde_json::Value> {
    let mut files = Vec::new();

    if !body.paths.is_empty() {
        for p in &body.paths {
            if !is_valid_path(p) {
                continue;
            }
            let path = std::path::Path::new(p);
            if path.is_file() && path.extension().is_some() {
                let entry = build_file_entry(path, p);
                files.push(serde_json::to_value(entry).unwrap_or_default());
            } else if path.is_dir() {
                scan_dir_recursive(path, &mut files);
            }
        }
    } else if !body.folders.is_empty() {
        let safe_folders: Vec<String> = body.folders.into_iter()
            .filter(|f| is_valid_path(f))
            .collect();
        if let Some(fmt) = OUTPUT_FORMATS.get(body.format_key.as_str()) {
            let result = FileScanner::scan_folders_info(&safe_folders, &fmt.ext);
            for (input, output) in &result.files {
                let entry = build_file_entry(std::path::Path::new(input), input);
                let mut val = serde_json::to_value(&entry).unwrap_or_default();
                val["output"] = serde_json::json!(output);
                files.push(val);
            }
        }
    }

    Json(serde_json::json!(files))
}

/// Looks up an [`OutputFormat`] by key, stripping a leading `.` if present
/// (e.g. both `"mp3"` and `".mp3"` resolve correctly).
fn resolve_format_key(raw: &str) -> Option<&'static crate::formats::OutputFormat> {
    let stripped = raw.strip_prefix('.').unwrap_or(raw);
    let mut buf = [0u8; 32];
    buf[0] = b'.';
    let bytes = stripped.as_bytes();
    let n = bytes.len().min(buf.len() - 1);
    buf[1..=n].copy_from_slice(&bytes[..n]);
    let key = std::str::from_utf8(&buf[..=n]).unwrap_or("");
    OUTPUT_FORMATS.get(key)
        .or_else(|| OUTPUT_FORMATS.get(stripped))
}

/// Builds `(input, output, format_key)` pairs for every file × format combination.
///
/// Iterates over all requested format keys and maps each source file to its
/// output path based on the output directory and suffix settings.
fn build_file_pairs(
    req: &StartConversionRequest,
    output_in_same_folder: bool,
    output_directory: &str,
    output_suffix: &str,
    format_keys: &[String],
) -> Vec<(String, String, String)> {
    let mut pairs = Vec::new();

    for raw_fmt in format_keys {
        let fmt = match resolve_format_key(raw_fmt) {
            Some(f) => f,
            None => continue,
        };
        let out_ext = fmt.ext.trim_start_matches('.');
        let normalized_key = raw_fmt.strip_prefix('.').unwrap_or(raw_fmt);

        if let Some(files_arr) = &req.files {
            for item in files_arr {
                let path_str = item.as_str()
                    .or_else(|| item.get("path").and_then(|v| v.as_str()));
                if let Some(path_str) = path_str {
                    if !is_valid_path(path_str) {
                        continue;
                    }
                    let p = std::path::Path::new(path_str);
                    if !p.is_file() {
                        continue;
                    }
                    let stem = p.file_stem().unwrap_or_default().to_string_lossy();
                    let new_filename = format!("{}{}.{}", stem, output_suffix, out_ext);

                    let out_path = if !output_in_same_folder && !output_directory.is_empty() {
                        let dir = std::path::Path::new(output_directory);
                        let _ = std::fs::create_dir_all(dir);
                        dir.join(&new_filename)
                    } else {
                        p.parent().unwrap_or(std::path::Path::new(".")).join(&new_filename)
                    };

                    let out_str = out_path.to_string_lossy().to_string();
                    if out_str != path_str {
                        pairs.push((path_str.to_string(), out_str, format!(".{}", normalized_key)));
                    }
                }
            }
        } else {
            let safe_folders: Vec<String> = req.folders.iter()
                .filter(|f| is_valid_path(f))
                .cloned()
                .collect();
            let raw_pairs = FileScanner::scan_folders(&safe_folders, out_ext);
            for (inp, out) in raw_pairs {
                pairs.push((inp, out, format!(".{}", normalized_key)));
            }
        }
    }

    pairs
}

/// `POST /api/start` — validates the request, builds file pairs, and starts
/// the multi-threaded FFmpeg conversion. Returns `{jobId: "current"}` on success.
async fn start_conversion(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartConversionRequest>,
) -> Json<serde_json::Value> {
    if let Err(e) = req.validate() {
        return Json(serde_json::json!({"success": false, "error": e}));
    }

    let sample_rate = req.effective_sample_rate();
    let channels = req.effective_channels();
    let volume = req.volume.clamp(-20, 20);
    let trim_start = req.effective_trim_start();
    let trim_end = req.effective_trim_end();
    let codec_copy = req.effective_codec_copy();
    let output_suffix = req.effective_output_suffix();
    let output_directory = req.effective_output_directory();

    // Determine target format keys
    let format_keys = if let Some(ref formats) = req.formats {
        if formats.is_empty() {
            vec![req.effective_format_key().to_string()]
        } else {
            formats.clone()
        }
    } else {
        vec![req.effective_format_key().to_string()]
    };

    let file_pairs = build_file_pairs(
        &req, req.output_in_same_folder, output_directory, output_suffix, &format_keys,
    );

    if file_pairs.is_empty() {
        return Json(serde_json::json!({"success": false, "error": "Nenhum arquivo para converter. Verifique se os arquivos existem e o formato de saida e diferente do formato de entrada."}));
    }

    let bit_rate = req.bit_rate.as_deref().unwrap_or("");

    let file_count = file_pairs.len();
    let converter = state.converter.read().await;
    let started = converter.start(
        file_pairs, sample_rate, channels, volume,
        trim_start, trim_end, &req.output_pattern, codec_copy,
        None, req.output_subfolder, req.max_output_size_mb,
        bit_rate,
    );
    drop(converter);

    if started {
        Json(serde_json::json!({"success": true, "jobId": "current", "count": file_count}))
    } else {
        Json(serde_json::json!({"success": false, "error": "Falha ao iniciar conversao (outro processo em andamento?)"}))
    }
}

/// `POST /api/cancel` — signals the converter to abort all in-progress jobs.
async fn cancel_conversion(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let converter = state.converter.read().await;
    converter.cancel();
    Json(serde_json::json!({"success": true}))
}

/// `GET /api/status` or `GET /api/status/:jobId` — returns the current conversion
/// progress, per-file statuses, and overall completion state.
async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let converter = state.converter.read().await;
    Json(serde_json::to_value(converter.get_status()).unwrap_or_default())
}

/// `POST /api/waveform` — generates 200-point waveform data for an audio file.
/// Validates the path and extension before delegating to [`AudioPreview`].
async fn get_waveform(
    Json(body): Json<AudioPathRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !is_valid_path(&body.path) || !has_audio_extension(&body.path) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = AudioPreview::get_waveform_data(&body.path, 200);
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// `POST /api/spectrum` — generates 32-point spectrum data for an audio file.
async fn get_spectrum(
    Json(body): Json<AudioPathRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !is_valid_path(&body.path) || !has_audio_extension(&body.path) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = AudioPreview::get_waveform_data(&body.path, 32);
    Ok(Json(serde_json::json!({
        "success": result.success,
        "spectrum": result.waveform,
    })))
}

/// `POST /api/open-folder` — opens a folder in the OS file explorer, or opens
/// a native folder-picker dialog when `mode` is `"select"`.
async fn open_folder(
    Json(body): Json<PathRequest>,
) -> Json<serde_json::Value> {
    if body.mode == "select" {
        let folder = rfd::AsyncFileDialog::new()
            .set_title("Selecionar pasta de destino")
            .pick_folder()
            .await;
        match folder {
            Some(p) => Json(serde_json::json!({"success": true, "path": p.path().to_string_lossy().to_string()})),
            None => Json(serde_json::json!({"success": false, "error": "Selecao cancelada"})),
        }
    } else {
        if let Err(e) = validate_path(&body.path) {
            return e;
        }
        let path = body.path.clone();
        let result = tokio::task::spawn_blocking(move || {
            let p = std::path::Path::new(&path);
            match p.metadata() {
                Ok(meta) if meta.is_dir() => {
                    let _ = open::that(&path);
                    serde_json::json!({"success": true})
                }
                Ok(_) => serde_json::json!({"success": false, "error": "Caminho nao e um diretorio"}),
                Err(_) => serde_json::json!({"success": false, "error": "Caminho nao existe"}),
            }
        }).await.unwrap_or_else(|_| serde_json::json!({"success": false, "error": "Erro interno"}));
        Json(result)
    }
}

/// `POST /api/clear-cache` — two-phase cache cleanup.
///
/// First call (`confirm: false`) returns the number of files that would be deleted.
/// Second call (`confirm: true`) actually deletes them from the temp directories.
async fn clear_cache(
    Json(body): Json<ClearCacheRequest>,
) -> Json<serde_json::Value> {
    let confirm = body.confirm;
    let result = tokio::task::spawn_blocking(move || {
        let temp_dirs = [
            std::env::temp_dir().join("trix_audio"),
            std::env::temp_dir().join("trix"),
        ];

        if !confirm {
            let count: u32 = temp_dirs.iter()
                .filter_map(|d| std::fs::read_dir(d).ok())
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .count() as u32;
            return serde_json::json!({
                "success": true,
                "requires_confirmation": true,
                "files_to_delete": count,
            });
        }

        let mut removed = 0u32;
        for dir in &temp_dirs {
            if dir.exists() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let _ = std::fs::remove_file(&path);
                            removed += 1;
                        }
                    }
                }
                let _ = std::fs::remove_dir_all(dir);
            }
        }
        serde_json::json!({"success": true, "removed": removed})
    }).await.unwrap_or_else(|_| serde_json::json!({"success": false, "error": "Erro interno"}));
    Json(result)
}

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn SHChangeNotify(w_event_id: i32, u_flags: u32, dw_item1: *const std::ffi::c_void, dw_item2: *const std::ffi::c_void);
}

#[cfg(target_os = "windows")]
async fn register_context_menu() -> Json<serde_json::Value> {
    let result = tokio::task::spawn_blocking(|| {
        use winreg::enums::*;
        use winreg::RegKey;

        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if exe_path.is_empty() {
            return serde_json::json!({"success": false, "error": "Nao foi possivel obter o caminho do executavel"});
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        let key_path = "Software\\Classes\\*\\shell\\TrixConvert";
        let shell_key = hkcu.create_subkey(key_path)
            .and_then(|(k, _)| {
                let _ = k.set_value("", &"Converter com Trix Audio");
                let _ = k.set_value("Icon", &exe_path);
                Ok(k)
            })
            .and_then(|k| {
                k.create_subkey("command")
                    .and_then(|(cmd, _)| {
                        let _ = cmd.set_value("", &format!("\"{}\" --convert \"%1\"", exe_path));
                        Ok(())
                    })
            });

        match shell_key {
            Ok(_) => {
                // Notify Explorer to update cache immediately
                unsafe {
                    SHChangeNotify(0x08000000, 0, std::ptr::null(), std::ptr::null());
                }
                serde_json::json!({"success": true})
            },
            Err(e) => serde_json::json!({"success": false, "error": format!("Falha ao registrar menu de contexto: {}", e)}),
        }
    }).await.unwrap_or_else(|_| serde_json::json!({"success": false, "error": "Erro interno"}));
    Json(result)
}

#[cfg(not(target_os = "windows"))]
async fn register_context_menu() -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": false, "error": "Not supported on this OS"}))
}

#[cfg(target_os = "windows")]
async fn unregister_context_menu() -> Json<serde_json::Value> {
    let result = tokio::task::spawn_blocking(|| {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut last_err = None;

        // Delete the new * key
        if let Err(e) = hkcu.delete_subkey_all("Software\\Classes\\*\\shell\\TrixConvert") {
            if e.kind() != std::io::ErrorKind::NotFound {
                last_err = Some(e);
            }
        }

        // Clean up legacy SystemFileAssociations keys if they exist
        let extensions = vec![
            ".wav", ".mp3", ".flac", ".m4a", ".aac", ".ogg", ".wma", ".opus", ".amr",
            ".mp4", ".mkv", ".avi", ".mov", ".webm", ".flv", ".mpeg", ".mpg"
        ];
        for ext in &extensions {
            let key_path = format!("Software\\Classes\\SystemFileAssociations\\{}\\shell\\TrixConvert", ext);
            let _ = hkcu.delete_subkey_all(&key_path);
        }

        match last_err {
            None => {
                // Notify Explorer to update cache immediately
                unsafe {
                    SHChangeNotify(0x08000000, 0, std::ptr::null(), std::ptr::null());
                }
                serde_json::json!({"success": true})
            },
            Some(e) => serde_json::json!({"success": false, "error": format!("Falha ao remover menu de contexto: {}", e)}),
        }
    }).await.unwrap_or_else(|_| serde_json::json!({"success": false, "error": "Erro interno"}));
    Json(result)
}

#[cfg(not(target_os = "windows"))]
async fn unregister_context_menu() -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": false, "error": "Not supported on this OS"}))
}

// ── Video Extractor Handlers ──

/// `POST /api/video/probe` — reads video metadata (codec, duration, resolution)
/// without decoding frames. Uses [`VideoExtractor::probe`] internally.
async fn probe_video(
    Json(body): Json<PathRequest>,
) -> Json<serde_json::Value> {
    if let Err(e) = validate_path(&body.path) {
        return e;
    }
    match VideoExtractor::probe(&body.path) {
        Ok(info) => Json(serde_json::to_value(info).unwrap_or_default()),
        Err(_) => Json(serde_json::json!({"success": false, "error": "Falha ao analisar video"})),
    }
}

/// `POST /api/video/extract` — extracts the audio track from a video file
/// using FFmpeg and saves it to the specified output path.
async fn extract_audio_from_video(
    Json(body): Json<ExtractAudioRequest>,
) -> Json<serde_json::Value> {
    if body.input.is_empty() || !is_valid_path(&body.input) {
        return Json(serde_json::json!({"success": false, "error": "Caminho de entrada invalido"}));
    }
    if body.output.is_empty() || !is_valid_path(&body.output) {
        return Json(serde_json::json!({"success": false, "error": "Caminho de saida invalido"}));
    }

    match VideoExtractor::extract_audio(&body.input, &body.output, &body.format, body.bitrate.as_deref()) {
        Ok(path) => Json(serde_json::json!({"success": true, "path": path})),
        Err(e) => Json(serde_json::json!({"success": false, "error": format!("Falha ao extrair áudio do vídeo: {}", e)})),
    }
}

// ── Window Control ──

/// `POST /api/window/close` — sends a close command to the tao window event loop.
async fn window_close() -> Json<serde_json::Value> {
    crate::send_window_cmd("close");
    Json(serde_json::json!({"success": true}))
}

/// `POST /api/window/minimize` — sends a minimize command to the tao window event loop.
async fn window_minimize() -> Json<serde_json::Value> {
    crate::send_window_cmd("minimize");
    Json(serde_json::json!({"success": true}))
}

/// `POST /api/window/maximize` — sends a maximize/restore command to the tao window event loop.
async fn window_maximize() -> Json<serde_json::Value> {
    crate::send_window_cmd("maximize");
    Json(serde_json::json!({"success": true}))
}

// ── File Picker & Drop Handler ──

/// `POST /api/pick-files` — opens the OS native file-picker dialog filtered
/// to supported audio and video extensions. Returns a JSON array of absolute paths.
async fn pick_files() -> Json<serde_json::Value> {
    let paths = rfd::AsyncFileDialog::new()
        .add_filter("Arquivos Suportados (Áudio/Vídeo)", &["wav", "mp3", "flac", "aac", "ogg", "opus", "m4a", "aiff", "alac", "wma", "ape", "dsf", "dff", "wv", "mp2", "m4b", "m4r", "amr", "dts", "ac3", "eac3", "mp4", "mkv", "avi", "mov", "flv", "webm", "mka", "ogv", "wmv", "3gp"])
        .add_filter("Áudio", &["wav", "mp3", "flac", "aac", "ogg", "opus", "m4a", "aiff", "alac", "wma", "ape", "dsf", "dff", "wv", "mp2", "m4b", "m4r", "amr", "dts", "ac3", "eac3"])
        .add_filter("Vídeo", &["mp4", "mkv", "avi", "mov", "flv", "webm", "mka", "ogv", "wmv", "3gp"])
        .add_filter("Todos os Arquivos", &["*"])
        .set_title("Selecionar arquivos de áudio/vídeo")
        .pick_files()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.path().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    Json(serde_json::json!(paths))
}

/// `GET /api/dropped-files` — drains and returns file paths that were
/// drag-and-dropped onto the native window (stored in [`DROPPED_FILES`]).
async fn get_dropped_files() -> Json<serde_json::Value> {
    let files = crate::DROPPED_FILES
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .drain(..)
        .collect::<Vec<_>>();
    Json(serde_json::json!(files))
}

// ── Crash Logs ──

/// `GET /api/crash-logs` — lists crash log files from the app logs directory.
async fn get_crash_logs() -> Json<serde_json::Value> {
    let logs = crate::crash_logger::get_crash_logs();
    Json(serde_json::json!({
        "success": true,
        "logs": logs,
    }))
}

/// Request body for `/api/crash-logs/read`.
#[derive(Deserialize)]
struct ReadCrashLogRequest {
    /// Absolute path to the crash log file to read.
    path: String,
}

/// `POST /api/crash-logs/read` — reads and returns the content of a crash log file.
async fn read_crash_log(Json(req): Json<ReadCrashLogRequest>) -> Json<serde_json::Value> {
    let content = crate::crash_logger::read_crash_log(&req.path);
    Json(serde_json::json!({
        "success": true,
        "content": content,
    }))
}

/// Request body for `/api/crash-logs/delete`.
#[derive(Deserialize)]
struct DeleteCrashLogRequest {
    /// Absolute path to the crash log file to delete.
    path: String,
}

/// `POST /api/crash-logs/delete` — deletes a crash log file by absolute path.
async fn delete_crash_log(Json(req): Json<DeleteCrashLogRequest>) -> Json<serde_json::Value> {
    match crate::crash_logger::delete_crash_log(&req.path) {
        Ok(()) => Json(serde_json::json!({"success": true})),
        Err(e) => Json(serde_json::json!({"success": false, "error": e})),
    }
}
