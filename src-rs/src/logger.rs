//! Conversion Logger
//!
//! Logs conversion sessions to human-readable `.log` files and structured
//! `.jsonl` files. Manages log rotation with 30-day retention.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, Duration};

/// A single log file entry with metadata for listing conversion sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub date: String,
}

/// Entrada de log estruturada para JSONL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonlLogEntry {
    pub timestamp: String,
    pub level: String,
    pub event: String,
    pub file: Option<String>,
    pub path: Option<String>,
    pub status: Option<String>,
    pub elapsed_secs: Option<f64>,
    pub error: Option<String>,
    pub input_size: Option<u64>,
    pub output_size: Option<u64>,
    pub compression_ratio: Option<f64>,
}

const LOG_RETENTION_DAYS: u64 = 30;

/// Manages conversion log files in `.log` and `.jsonl` formats.
pub struct ConversionLogger {
    current_log: Option<PathBuf>,
    current_jsonl_log: Option<PathBuf>,
    entries: Vec<LogEntry>,
    logs_dir: PathBuf,
}

impl ConversionLogger {
    /// Creates a new `ConversionLogger`, ensuring the logs directory exists
    /// and removing log files older than `LOG_RETENTION_DAYS`.
    pub fn new() -> Self {
        let logs_dir = crate::portable::Portable::logs_dir();
        let _ = std::fs::create_dir_all(&logs_dir);

        let logger = Self {
            current_log: None,
            current_jsonl_log: None,
            entries: Vec::new(),
            logs_dir,
        };
        logger.cleanup_old_logs();
        logger
    }

    fn cleanup_old_logs(&self) {
        let cutoff = SystemTime::now().checked_sub(Duration::from_secs(LOG_RETENTION_DAYS * 86400));
        if let Some(cutoff) = cutoff {
            if let Ok(entries) = std::fs::read_dir(&self.logs_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if modified < cutoff && path.file_name().unwrap_or_default().to_string_lossy().starts_with("conversion_") {
                                let _ = std::fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Opens a new `.log` + `.jsonl` pair for a conversion batch.
    ///
    /// Returns a timestamp string (used as the job identifier).
    /// Clears any in-memory entries from previous batches.
    pub fn start_log(&mut self, format_key: &str, file_count: usize, settings: &str) -> String {
        let _ = std::fs::create_dir_all(&self.logs_dir);
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let log_path = self.logs_dir.join(format!("conversion_{}.log", timestamp));
        let jsonl_path = self.logs_dir.join(format!("conversion_{}.jsonl", timestamp));

        let header = format!(
            "============================================================\n\
             AudioMaster Pro - Log de Conversao\n\
             ============================================================\n\
             Data: {}\n\
             Formato: {}\n\
             Total de arquivos: {}\n\
             Configuracoes:\n{}\n\
             ============================================================\n\n",
            chrono::Local::now().format("%d/%m/%Y %H:%M:%S"),
            format_key,
            file_count,
            settings,
        );

        let _ = std::fs::write(&log_path, header);
        
        // Write JSONL header
        let jsonl_header = JsonlLogEntry {
            timestamp: chrono::Local::now().to_rfc3339(),
            level: "INFO".into(),
            event: "conversion_started".into(),
            file: None,
            path: None,
            status: None,
            elapsed_secs: None,
            error: None,
            input_size: None,
            output_size: None,
            compression_ratio: None,
        };
        let _ = std::fs::write(&jsonl_path, serde_json::to_string(&jsonl_header).unwrap_or_default() + "\n");

        self.current_log = Some(log_path);
        self.current_jsonl_log = Some(jsonl_path);
        self.entries.clear();
        timestamp
    }

    /// Appends one file's result to both the human-readable `.log` and the
    /// structured `.jsonl` file. Computes the compression ratio when both
    /// `input_size` and `output_size` are non-zero.
    pub fn log_file(&mut self, filename: &str, input_path: &str, _output_path: &str, status: &str, elapsed: f64, error: &str, input_size: u64, output_size: u64) {
        let entry = LogEntry {
            name: filename.to_string(),
            path: input_path.to_string(),
            size: 0,
            date: chrono::Local::now().format("%H:%M:%S").to_string(),
        };
        self.entries.push(entry);

        let ratio = if input_size > 0 && output_size > 0 {
            output_size as f64 / input_size as f64
        } else {
            0.0
        };

        // Write human-readable log
        if let Some(log_path) = &self.current_log {
            let mut line = format!(
                "[{}] {:>10} | {:<40} | {:.1}s",
                chrono::Local::now().format("%H:%M:%S"),
                status.to_uppercase(),
                filename,
                elapsed
            );

            if !error.is_empty() {
                let err_display = if error.len() > 100 { &error[..100] } else { error };
                line += &format!(" | ERRO: {}", err_display);
            }

            if input_size > 0 && output_size > 0 {
                line += &format!(
                    " | {} -> {} ({:.0}%)",
                    crate::formats::human_size(input_size),
                    crate::formats::human_size(output_size),
                    ratio * 100.0
                );
            }

            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{}", line)
                });
        }

        // Write JSONL log
        if let Some(jsonl_path) = &self.current_jsonl_log {
            let jsonl_entry = JsonlLogEntry {
                timestamp: chrono::Local::now().to_rfc3339(),
                level: if status == "error" { "ERROR".into() } else { "INFO".into() },
                event: "file_processed".into(),
                file: Some(filename.into()),
                path: Some(input_path.into()),
                status: Some(status.into()),
                elapsed_secs: Some(elapsed),
                error: if error.is_empty() { None } else { Some(error.into()) },
                input_size: if input_size > 0 { Some(input_size) } else { None },
                output_size: if output_size > 0 { Some(output_size) } else { None },
                compression_ratio: if input_size > 0 && output_size > 0 { Some(ratio) } else { None },
            };
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(jsonl_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{}", serde_json::to_string(&jsonl_entry).unwrap_or_default())
                });
        }
    }

    /// Appends the batch summary (total, converted, failed, timing) to the
    /// current `.log` file and closes the session.
    pub fn finish_log(&self, total: usize, converted: usize, failed: usize, elapsed: f64) {
        if let Some(log_path) = &self.current_log {
            let avg = if total > 0 { elapsed / total as f64 } else { 0.0 };
            let summary = format!(
                "\n============================================================\n\
                 RESUMO\n\
                 Total: {} | Sucesso: {} | Falhas: {}\n\
                 Tempo total: {:.1}s\n\
                 Tempo medio por arquivo: {:.1}s\n\
                 Log salvo em: {}\n\
                 =============================================================\n",
                total, converted, failed, elapsed, avg,
                log_path.display()
            );
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    write!(f, "{}", summary)
                });
        }
    }

    /// Returns all `conversion_*.log` files in the logs directory,
    /// sorted newest-first, with their size and last-modified date.
    pub fn get_logs(&self) -> Vec<LogEntry> {
        let _ = std::fs::create_dir_all(&self.logs_dir);
        let mut logs = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.logs_dir) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().file_name().unwrap_or_default().to_string_lossy().starts_with("conversion_"))
                .collect();
            files.sort_by(|a, b| b.path().cmp(&a.path()));

            for entry in files {
                let meta = entry.metadata().ok();
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let date = meta
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| {
                        let dt: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default();
                        dt.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "unknown".into());

                logs.push(LogEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    size,
                    date,
                });
            }
        }

        logs
    }

    /// Reads and returns the contents of a log file.
    /// Only files directly inside `logs_dir` are served (path-traversal guard).
    pub fn read_log(&self, log_path: &str) -> String {
        let p = std::path::Path::new(log_path);
        if p.parent() == Some(&self.logs_dir) {
            std::fs::read_to_string(p).unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// Deletes a log file.
    /// Only files directly inside `logs_dir` are deleted (path-traversal guard).
    pub fn delete_log(&self, log_path: &str) -> Result<(), String> {
        let p = std::path::Path::new(log_path);
        if p.parent() == Some(&self.logs_dir) {
            std::fs::remove_file(p).map_err(|e| e.to_string())
        } else {
            Err("Arquivo nao encontrado".into())
        }
    }
}
