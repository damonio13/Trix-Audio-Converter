//! Device-specific audio presets (phones, headphones, speakers).
//!
//! Contains optimized encoding and EQ profiles tailored for playback
//! on various consumer devices and listening environments.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Encoding preset optimized for a specific playback device.
pub struct DevicePreset {
    pub name: String,
    pub device: String,
    pub format: String,
    pub codec: String,
    pub bitrate: String,
    pub sample_rate: String,
    pub channels: String,
    pub profile: String,
}

impl DevicePreset {
    /// Creates a new device preset with default sample rate and channels.
    pub fn new(name: &str, device: &str, format: &str, codec: &str, bitrate: &str) -> Self {
        Self {
            name: name.to_string(),
            device: device.to_string(),
            format: format.to_string(),
            codec: codec.to_string(),
            bitrate: bitrate.to_string(),
            sample_rate: "44100".to_string(),
            channels: "2".to_string(),
            profile: "".to_string(),
        }
    }
}

/// Predefined audio encoding presets for various devices and platforms.
pub struct DevicePresets;

impl DevicePresets {
    /// Returns all device presets for every supported platform.
    pub fn get_all() -> Vec<DevicePreset> {
        vec![
            // Apple
            DevicePreset {
                name: "iPhone (AAC)".into(),
                device: "iPhone".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "256k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            DevicePreset::new("iPhone (Lossless ALAC)", "iPhone", "m4a", "alac", "0"),
            DevicePreset {
                name: "iPad".into(),
                device: "iPad".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "256k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            DevicePreset {
                name: "Mac (AAC)".into(),
                device: "Mac".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "320k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_main".into(),
            },
            DevicePreset::new("Mac (Lossless ALAC)", "Mac", "m4a", "alac", "0"),
            // Android
            DevicePreset {
                name: "Android (AAC 256k)".into(),
                device: "Android".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "256k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            DevicePreset::new("Android (MP3 320k)", "Android", "mp3", "libmp3lame", "320k"),
            DevicePreset {
                name: "Android (Opus)".into(),
                device: "Android".into(),
                format: "ogg".into(),
                codec: "libopus".into(),
                bitrate: "128k".into(),
                sample_rate: "48000".into(),
                channels: "2".into(),
                profile: "".into(),
            },
            // Sony PlayStation
            DevicePreset {
                name: "PS5 (AAC)".into(),
                device: "PlayStation 5".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "256k".into(),
                sample_rate: "48000".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            DevicePreset::new("PS5 (FLAC)", "PlayStation 5", "flac", "flac", "0"),
            DevicePreset {
                name: "PS4 (AAC)".into(),
                device: "PlayStation 4".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "192k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            // Xbox
            DevicePreset {
                name: "Xbox (AAC)".into(),
                device: "Xbox".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "256k".into(),
                sample_rate: "48000".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            DevicePreset::new("Xbox (WMA)", "Xbox", "wma", "wmav2", "192k"),
            // Nintendo
            DevicePreset {
                name: "Switch (AAC)".into(),
                device: "Nintendo Switch".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "128k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            // Streaming
            DevicePreset::new("Spotify Quality", "Streaming", "ogg", "libvorbis", "160k"),
            DevicePreset::new("YouTube Quality", "Streaming", "mp3", "libmp3lame", "192k"),
            DevicePreset::new("TikTok/Instagram", "Streaming", "mp3", "libmp3lame", "128k"),
            // Audiophile
            DevicePreset {
                name: "Audiophile FLAC".into(),
                device: "Hi-Fi".into(),
                format: "flac".into(),
                codec: "flac".into(),
                bitrate: "0".into(),
                sample_rate: "96000".into(),
                channels: "2".into(),
                profile: "".into(),
            },
            DevicePreset {
                name: "Audiophile DSD".into(),
                device: "Hi-Fi".into(),
                format: "dsf".into(),
                codec: "dsf".into(),
                bitrate: "0".into(),
                sample_rate: "2822400".into(),
                channels: "2".into(),
                profile: "".into(),
            },
            // Voice
            DevicePreset {
                name: "Voice Recording".into(),
                device: "Voice".into(),
                format: "mp3".into(),
                codec: "libmp3lame".into(),
                bitrate: "64k".into(),
                sample_rate: "22050".into(),
                channels: "1".into(),
                profile: "".into(),
            },
            DevicePreset {
                name: "Podcast".into(),
                device: "Voice".into(),
                format: "mp3".into(),
                codec: "libmp3lame".into(),
                bitrate: "128k".into(),
                sample_rate: "44100".into(),
                channels: "1".into(),
                profile: "".into(),
            },
            // Legacy
            DevicePreset {
                name: "iPod (AAC 128k)".into(),
                device: "iPod".into(),
                format: "m4a".into(),
                codec: "aac".into(),
                bitrate: "128k".into(),
                sample_rate: "44100".into(),
                channels: "2".into(),
                profile: "aac_low".into(),
            },
            DevicePreset::new("Walkman (ATRAC)", "Walkman", "oma", "atrac3", "132k"),
        ]
    }

    /// Returns the unique device category names from all presets.
    pub fn get_categories() -> Vec<String> {
        let presets = Self::get_all();
        let mut categories: Vec<String> = presets.iter()
            .map(|p| p.device.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// Returns all presets for a specific device category.
    pub fn get_by_category(category: &str) -> Vec<DevicePreset> {
        Self::get_all().into_iter()
            .filter(|p| p.device == category)
            .collect()
    }
}
