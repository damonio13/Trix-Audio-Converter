//! Ringtone cutter with preset durations
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Configuration for ringtone cutting.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RingtoneConfig {
    pub start: f64,
    pub duration: f64,
    pub fade_in: f64,
    pub fade_out: f64,
    pub format: String,
    pub bitrate: String,
}

impl Default for RingtoneConfig {
    fn default() -> Self {
        Self {
            start: 0.0,
            duration: 30.0,
            fade_in: 0.5,
            fade_out: 0.5,
            format: "m4a".into(),
            bitrate: "192k".into(),
        }
    }
}

/// Cuts audio into ringtones with configurable durations and fades.
pub struct RingtoneCutter;

impl RingtoneCutter {
    /// Cuts a segment from the input file according to the ringtone config.
    pub fn cut(input: &str, output: &str, config: &RingtoneConfig) -> Result<String, String> {
        let start_str = crate::utils::format_mm_ss_ms(config.start, 3);
        let dur_str = config.duration.to_string();

        let mut args = vec![
            "-i", input,
            "-ss", &start_str,
            "-t", &dur_str,
        ];

        let probe = crate::utils::ffprobe_json(input)?;
        let info = crate::utils::parse_audio_probe(&probe)?;
        let total_samples = (config.duration * info.sample_rate as f64) as u64;
        let fade_filters = crate::utils::build_fade_filters(total_samples, info.sample_rate, config.fade_in, config.fade_out);

        let filter_str = fade_filters.join(",");
        if !filter_str.is_empty() {
            args.extend(["-af", &filter_str]);
        }

        if let Some(c) = crate::utils::codec_for_format(&config.format) {
            args.extend(["-codec:a", c.codec, "-b:a", &config.bitrate]);
        } else {
            args.extend(["-codec:a", "aac", "-b:a", &config.bitrate]);
        }

        args.extend(["-y", "--", output]);

        crate::utils::run_ffmpeg_raw_checked(&args)?;

        Ok(output.to_string())
    }

    /// Creates an iPhone-compatible ringtone (`.m4r`, max 30 s) from `input`
    /// starting at `start` seconds.
    pub fn create_iphone_ringtone(input: &str, output: &str, start: f64) -> Result<String, String> {
        let config = RingtoneConfig {
            start,
            duration: 30.0,
            fade_in: 0.0,
            fade_out: 0.3,
            format: "m4a".into(),
            bitrate: "256k".into(),
        };
        Self::cut(input, output, &config)
    }

    /// Creates an Android-compatible ringtone (`.ogg`) from `input`
    /// starting at `start` seconds.
    pub fn create_android_ringtone(input: &str, output: &str, start: f64) -> Result<String, String> {
        let config = RingtoneConfig {
            start,
            duration: 30.0,
            fade_in: 0.0,
            fade_out: 0.5,
            format: "mp3".into(),
            bitrate: "192k".into(),
        };
        Self::cut(input, output, &config)
    }

    /// Creates a short notification sound (max 3 s) from `input`
    /// starting at `start` seconds.
    pub fn create_notification(input: &str, output: &str, start: f64) -> Result<String, String> {
        let config = RingtoneConfig {
            start,
            duration: 10.0,
            fade_in: 0.0,
            fade_out: 0.3,
            format: "ogg".into(),
            bitrate: "128k".into(),
        };
        Self::cut(input, output, &config)
    }

    /// Creates a loopable alarm sound from `input` starting at `start` seconds.
    pub fn create_alarm(input: &str, output: &str, start: f64) -> Result<String, String> {
        let config = RingtoneConfig {
            start,
            duration: 60.0,
            fade_in: 1.0,
            fade_out: 1.0,
            format: "mp3".into(),
            bitrate: "192k".into(),
        };
        Self::cut(input, output, &config)
    }

    /// Returns preset ringtone types as `(id, display_name, max_duration_secs)` tuples.
    pub fn get_presets() -> &'static [(&'static str, &'static str, f64)] {
        static STATIC: LazyLock<&[(&str, &str, f64)]> = LazyLock::new(|| {
            &[
                ("iPhone Ringtone", "m4a", 30.0),
                ("Android Ringtone", "mp3", 30.0),
                ("Notification", "ogg", 10.0),
                ("Alarm", "mp3", 60.0),
                ("WhatsApp Status", "mp3", 30.0),
                ("Instagram Story", "mp3", 15.0),
                ("TikTok Sound", "mp3", 15.0),
            ]
        });
        &*STATIC
    }
}
