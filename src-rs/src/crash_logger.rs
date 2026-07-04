//! Crash Logger
//!
//! Captures panics and writes crash reports to text files in the logs directory.
//! Includes system information, stack traces, and auto-cleanup of old logs.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::path::PathBuf;
use std::panic;
use std::sync::Once;
use chrono::Local;

static INSTALL_HOOK: Once = Once::new();

/// System information for crash reports
struct SystemInfo {
    os: String,
    arch: String,
    hostname: String,
    app_version: String,
}

impl SystemInfo {
    fn collect() -> Self {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "unknown".into());
        let app_version = env!("CARGO_PKG_VERSION").to_string();
        Self { os, arch, hostname, app_version }
    }

    fn display(&self) -> String {
        format!(
            "SO: {} {}\nHostname: {}Versao do app: {}",
            self.os, self.arch, self.hostname, self.app_version
        )
    }
}

/// Get the crash logs directory
fn crash_logs_dir() -> PathBuf {
    crate::portable::Portable::logs_dir().join("crashes")
}

/// Keep only the last N crash logs
fn cleanup_old_crash_logs(dir: &PathBuf, max_files: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "txt"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let modified = meta.modified().ok()?;
            Some((e.path(), modified))
        })
        .collect();

    files.sort_by(|a, b| b.1.cmp(&a.1));

    for (path, _) in files.into_iter().skip(max_files) {
        let _ = std::fs::remove_file(&path);
    }
}

/// Install the panic hook to capture crashes
pub fn install() {
    INSTALL_HOOK.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            // Collect system info
            let sys = SystemInfo::collect();

            // Extract panic payload
            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Erro desconhecido".to_string()
            };

            // Extract location
            let location = info.location()
                .map(|loc| {
                    format!(
                        "Arquivo: {}\nLinha: {}\nColuna: {}",
                        loc.file(), loc.line(), loc.column()
                    )
                })
                .unwrap_or_else(|| "Localizacao desconhecida".to_string());

            // Extract thread name
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("unnamed");

            // Build the crash report
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let report = format!(
                "============================================================\n\
                 TRIX AUDIO CONVERTER - RELATORIO DE CRASH\n\
                 ============================================================\n\
                 Data: {}\n\
                 \n\
                 SISTEMA\n\
                 -------\n\
                 {}\n\
                 \n\
                 CRASH\n\
                 -------\n\
                 Thread: {}\n\
                 {}\n\
                 \n\
                 ERRO\n\
                 -------\n\
                 {}\n\
                 \n\
                 STACK TRACE\n\
                 -------\n\
                 {:?}\n\
                 =============================================================\n",
                timestamp,
                sys.display(),
                thread_name,
                location,
                payload,
                info,
            );

            // Write to crash log file
            let dir = crash_logs_dir();
            let _ = std::fs::create_dir_all(&dir);
            let filename = format!("crash_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
            let path = dir.join(&filename);
            let _ = std::fs::write(&path, &report);

            // Cleanup old logs (keep last 20)
            cleanup_old_crash_logs(&dir, 20);

            // Also print to stderr
            eprintln!("{}", report);

            // Call the default hook (prints to stderr with backtrace if RUST_BACKTRACE=1)
            default_hook(info);
        }));
    });
}

/// Get all crash logs as a list of { name, path, date, size }
pub fn get_crash_logs() -> Vec<CrashLogEntry> {
    let dir = crash_logs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut logs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "txt"))
            .collect();
        files.sort_by(|a, b| b.path().cmp(&a.path()));

        for entry in files {
            let meta = entry.metadata().ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let date = meta
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                .map(|d| {
                    let dt: chrono::DateTime<chrono::Utc> =
                        chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default();
                    dt.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M").to_string()
                })
                .unwrap_or_else(|| "unknown".into());

            logs.push(CrashLogEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                size,
                date,
            });
        }
    }

    logs
}

/// Read a crash log file (with path traversal protection)
pub fn read_crash_log(log_path: &str) -> String {
    let dir = crash_logs_dir();
    let p = std::path::Path::new(log_path);
    if p.parent() == Some(&dir) {
        std::fs::read_to_string(p).unwrap_or_default()
    } else {
        String::new()
    }
}

/// Delete a crash log file (with path traversal protection)
pub fn delete_crash_log(log_path: &str) -> Result<(), String> {
    let dir = crash_logs_dir();
    let p = std::path::Path::new(log_path);
    if p.parent() == Some(&dir) {
        std::fs::remove_file(p).map_err(|e| e.to_string())
    } else {
        Err("Arquivo nao encontrado".into())
    }
}

/// A single crash log entry with metadata.
#[derive(serde::Serialize)]
pub struct CrashLogEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub date: String,
}
