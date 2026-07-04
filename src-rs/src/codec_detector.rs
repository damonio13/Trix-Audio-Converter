//! Audio codec detection and identification.
//!
//! Identifies audio codecs by inspecting file headers, magic bytes,
//! and container format signatures for accurate format classification.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Information about available FFmpeg codecs and formats.
pub struct CodecInfo {
    pub encoders: HashMap<String, String>,
    pub decoders: HashMap<String, String>,
    pub formats: HashMap<String, String>,
    pub encoder_count: usize,
    pub decoder_count: usize,
    pub format_count: usize,
    pub ffmpeg_version: String,
}

/// Detects available audio codecs and formats from the system FFmpeg installation.
pub struct CodecDetector;

fn cached_detect() -> &'static CodecInfo {
    static INSTANCE: OnceLock<CodecInfo> = OnceLock::new();
    INSTANCE.get_or_init(|| CodecDetector::build_info())
}

impl CodecDetector {
    fn build_info() -> CodecInfo {
        let encoders = Self::parse_codecs("encoders");
        let decoders = Self::parse_codecs("decoders");
        let formats = Self::parse_formats();

        CodecInfo {
            encoder_count: encoders.len(),
            decoder_count: decoders.len(),
            format_count: formats.len(),
            ffmpeg_version: Self::get_version(),
            encoders,
            decoders,
            formats,
        }
    }

    /// Returns cached codec information, building it on first call.
    pub fn detect() -> &'static CodecInfo {
        cached_detect()
    }

    fn parse_ffmpeg_list(args: &[&str], column_offset: usize) -> HashMap<String, String> {
        let output = Command::new("ffmpeg")
            .args(args)
            .output();

        let mut items = HashMap::new();
        if let Ok(out) = output {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let line = line.trim_start();
                if line.len() > column_offset + 1 && line.as_bytes()[0].is_ascii_whitespace() {
                    let rest = &line[column_offset..];
                    if let Some(pos) = rest.find(|c: char| !c.is_whitespace()) {
                        let rest = &rest[pos..];
                        if let Some(space_pos) = rest.find(char::is_whitespace) {
                            let name = &rest[..space_pos];
                            let desc = rest[space_pos..].trim();
                            items.insert(name.to_string(), desc.to_string());
                        }
                    }
                }
            }
        }
        items
    }

    fn parse_codecs(mode: &str) -> HashMap<String, String> {
        Self::parse_ffmpeg_list(&["-hide_banner", &format!("-{}", mode)], 6)
    }

    fn parse_formats() -> HashMap<String, String> {
        Self::parse_ffmpeg_list(&["-hide_banner", "-formats"], 2)
    }

    fn get_version() -> String {
        Command::new("ffmpeg")
            .args(["-version"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| l.to_string()))
            .unwrap_or_else(|| "unknown".into())
    }

    /// Checks if FFmpeg has an encoder for the specified codec.
    pub fn has_encoder(codec_name: &str) -> bool {
        Self::detect().encoders.contains_key(codec_name)
    }

    /// Checks if FFmpeg has a decoder for the specified codec.
    pub fn has_decoder(codec_name: &str) -> bool {
        Self::detect().decoders.contains_key(codec_name)
    }
}
