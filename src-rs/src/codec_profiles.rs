//! Codec-specific encoding profiles and presets.
//!
//! Defines encoding parameter presets for various audio codecs including
//! bitrate, sample rate, VBR settings, and quality targets.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Encoding profile defining codec, bitrate, sample rate, and VBR settings.
pub struct CodecProfile {
    pub name: String,
    pub format: String,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub vbr: bool,
    pub vbr_quality: u32,
    pub extra_args: Vec<String>,
}

impl CodecProfile {
    /// Creates a simple encoding profile with default settings.
    pub fn simple(name: &str, format: &str, bitrate: u32) -> Self {
        Self {
            name: name.to_string(),
            format: format.to_string(),
            bitrate,
            sample_rate: 44100,
            channels: 2,
            vbr: false,
            vbr_quality: 0,
            extra_args: vec![],
        }
    }
}

/// Manages saving, loading, and listing of codec encoding profiles.
pub struct ProfileManager {
    store: crate::utils::JsonStore<CodecProfile>,
}

impl ProfileManager {
    /// Creates a new profile manager.
    pub fn new() -> Self {
        let dir = crate::portable::Portable::data_dir().join("profiles");
        let _ = std::fs::create_dir_all(&dir);

        Self {
            store: crate::utils::JsonStore::new(dir, 65536),
        }
    }

    /// Saves an encoding profile to persistent storage.
    pub fn save(&self, profile: &CodecProfile) -> Result<(), String> {
        self.store.save(&profile.name, profile)
    }

    /// Loads an encoding profile by name, sanitizing its fields.
    pub fn load(&self, name: &str) -> Result<CodecProfile, String> {
        let mut profile: CodecProfile = self.store.load(name)?;
        profile.name = profile.name.chars().take(200).collect();
        profile.extra_args.retain(|a| {
            !a.is_empty() && a.len() <= 200 && !a.starts_with('-')
        });
        if profile.extra_args.len() > 20 {
            profile.extra_args.truncate(20);
        }
        Ok(profile)
    }

    /// Lists all saved profile names.
    pub fn list(&self) -> Vec<String> {
        self.store.list()
    }

    /// Deletes an encoding profile by name.
    pub fn delete(&self, name: &str) -> Result<(), String> {
        self.store.delete(name)
    }

    /// Returns the built-in default encoding profiles.
    pub fn get_defaults() -> Vec<CodecProfile> {
        vec![
            CodecProfile::simple("MP3 128kbps", "mp3", 128),
            CodecProfile::simple("MP3 320kbps", "mp3", 320),
            CodecProfile {
                name: "MP3 VBR V0".into(),
                format: "mp3".into(),
                bitrate: 0,
                sample_rate: 44100,
                channels: 2,
                vbr: true,
                vbr_quality: 0,
                extra_args: vec!["-q:a".to_string(), "0".to_string()],
            },
            CodecProfile::simple("AAC 256kbps", "aac", 256),
            CodecProfile {
                name: "FLAC Lossless".into(),
                format: "flac".into(),
                bitrate: 0,
                sample_rate: 44100,
                channels: 2,
                vbr: false,
                vbr_quality: 0,
                extra_args: vec!["-compression_level".to_string(), "8".to_string()],
            },
            CodecProfile {
                name: "Opus 128kbps".into(),
                format: "opus".into(),
                bitrate: 128,
                sample_rate: 48000,
                channels: 2,
                vbr: false,
                vbr_quality: 0,
                extra_args: vec![],
            },
            CodecProfile {
                name: "Ogg Vorbis q5".into(),
                format: "ogg".into(),
                bitrate: 0,
                sample_rate: 44100,
                channels: 2,
                vbr: true,
                vbr_quality: 5,
                extra_args: vec!["-q:a".to_string(), "5".to_string()],
            },
            CodecProfile {
                name: "WAV 16bit".into(),
                format: "wav".into(),
                bitrate: 0,
                sample_rate: 44100,
                channels: 2,
                vbr: false,
                vbr_quality: 0,
                extra_args: vec!["-sample_fmt".to_string(), "s16".to_string()],
            },
        ]
    }
}
