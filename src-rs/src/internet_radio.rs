//! Internet radio streaming and recording.
//!
//! Connects to internet radio streams via stream URLs, supports
//! auto-reconnection, and records streams to local audio files.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

fn is_valid_hostname(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 253
        && s.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == ':')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.starts_with('.')
        && !s.ends_with('.')
}

fn is_valid_mount(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.')
}

fn is_valid_port(port: u16) -> bool {
    port >= 1024
}

fn is_valid_bitrate(bitrate: u32) -> bool {
    bitrate >= 32 && bitrate <= 512
}

fn is_valid_password(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars().all(|c| c.is_alphanumeric() || "!@#$%^&*()_+-=".contains(c))
}

fn validate_stream_params(input: &str, port: u16, bitrate: u32) -> Result<(), String> {
    if !crate::utils::is_safe_path(input) { return Err("Caminho invalido".into()); }
    if !is_valid_port(port) { return Err("Porta invalida".into()); }
    if !is_valid_bitrate(bitrate) { return Err("Bitrate invalido".into()); }
    Ok(())
}

/// Internet radio stream manager for local and Icecast broadcasting.
pub struct InternetRadio {
    child: Arc<Mutex<Option<Child>>>,
}

impl InternetRadio {
    /// Creates a new InternetRadio instance.
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts a local HTTP audio stream on the specified port.
    pub fn start_stream(
        &self,
        input: &str,
        port: u16,
        _format: &str,
        bitrate: u32,
    ) -> Result<String, String> {
        validate_stream_params(input, port, bitrate)?;

        let url = format!("http://127.0.0.1:{}", port);

        let child = Command::new("ffmpeg")
            .args([
                "-re",
                "-i", input,
                "-c:a", "libmp3lame",
                "-b:a", &format!("{}k", bitrate),
                "-f", "mp3",
                "-listen", "1",
                "--",
                &url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "FFmpeg não encontrado. Instale o FFmpeg e adicione ao PATH.".to_string()
                } else {
                    format!("Falha ao iniciar stream: {}", e)
                }
            })?;

        *self.child.lock().unwrap() = Some(child);

        Ok(format!("Stream iniciado em {}", url))
    }

    /// Stops the currently running audio stream.
    pub fn stop_stream(&self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
            }
            *guard = None;
        }
    }

    /// Returns true if an audio stream is currently running.
    pub fn is_streaming(&self) -> bool {
        if let Ok(mut guard) = self.child.lock() {
            guard.as_mut().map_or(false, |c| matches!(c.try_wait(), Ok(None)))
        } else {
            false
        }
    }

    /// Starts streaming to an Icecast server with authentication.
    pub fn start_icecast(
        &self,
        input: &str,
        server: &str,
        port: u16,
        password: &str,
        mount: &str,
        bitrate: u32,
    ) -> Result<String, String> {
        validate_stream_params(input, port, bitrate)?;
        if !is_valid_hostname(server) {
            return Err(format!("Servidor invalido: '{}'. Use hostname ou IP valido.", server));
        }
        if !is_valid_password(password) {
            return Err("Senha invalida. Use apenas alfanumericos e !@#$%^&*()_+-=".into());
        }
        if !is_valid_mount(mount) {
            return Err(format!("Mount point invalido: '{}'. Use apenas alfanumericos, /, - ou _.", mount));
        }

        let icecast_url = format!("icecast://source:{}@{}:{}/{}", password, server, port, mount);
        let display_url = format!("http://{}:{}/{}", server, port, mount);

        let child = Command::new("ffmpeg")
            .args([
                "-re",
                "-i", input,
                "-c:a", "libmp3lame",
                "-b:a", &format!("{}k", bitrate),
                "-f", "mp3",
                "--", &icecast_url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "FFmpeg não encontrado. Instale o FFmpeg e adicione ao PATH.".to_string()
                } else {
                    format!("Falha ao iniciar stream Icecast: {}", e)
                }
            })?;

        *self.child.lock().unwrap() = Some(child);

        // Verify connection after a short delay
        std::thread::sleep(Duration::from_millis(500));
        if let Some(ref mut child) = *self.child.lock().unwrap() {
            if let Ok(Some(exit_status)) = child.try_wait() {
                if !exit_status.success() {
                    return Err(format!(
                        "Falha ao conectar no Icecast em {}. Verifique se o servidor está acessível e a senha está correta.",
                        display_url
                    ));
                }
            }
        }

        Ok(format!("Stream Icecast iniciado em {}", display_url))
    }

    /// Returns supported streaming formats with default bitrates.
    pub fn get_formats() -> &'static [(&'static str, &'static str, u32)] {
        static STATIC: LazyLock<&[(&str, &str, u32)]> = LazyLock::new(|| {
            &[
                ("mp3", "MP3", 128),
                ("aac", "AAC", 128),
                ("ogg", "Ogg Vorbis", 128),
                ("flac", "FLAC (lossless)", 0),
            ]
        });
        &*STATIC
    }

    /// Returns available bitrate options for streaming.
    pub fn get_bitrates() -> &'static [(u32, &'static str)] {
        static STATIC: LazyLock<&[(u32, &str)]> = LazyLock::new(|| {
            &[
                (64, "64 kbps (voz)"),
                (96, "96 kbps"),
                (128, "128 kbps (padrao)"),
                (192, "192 kbps"),
                (256, "256 kbps"),
                (320, "320 kbps (alta qualidade)"),
            ]
        });
        &*STATIC
    }
}
