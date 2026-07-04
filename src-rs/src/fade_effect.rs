//! Fade in/out effects for audio tracks.
//!
//! Applies linear, exponential, or logarithmic fade curves to the
//! beginning and end of audio segments.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::sync::LazyLock;

/// Fade in/out effect processor for audio tracks.
pub struct FadeEffect;

impl FadeEffect {
    /// Applies fade in and fade out effects with specified durations.
    pub fn apply_fade(
        input: &str,
        output: &str,
        fade_in: f64,
        fade_out: f64,
    ) -> Result<String, String> {
        let probe = crate::utils::ffprobe_json(input)?;
        let info = crate::utils::parse_audio_probe(&probe)?;
        let total_samples = (info.duration * info.sample_rate as f64) as u64;
        let filters = crate::utils::build_fade_filters(total_samples, info.sample_rate, fade_in, fade_out);

        if filters.is_empty() {
            return Err("Nenhum fade especificado".into());
        }

        crate::utils::run_ffmpeg_af(input, output, &filters.join(","))
    }

    /// Applies a fade at a specific time position with the given type and duration.
    pub fn apply_fade_at(
        input: &str,
        output: &str,
        fade_type: &str,
        start: f64,
        duration: f64,
    ) -> Result<String, String> {
        let af = match fade_type {
            "in" => format!("afade=t=in:st={}:d={}", start, duration),
            "out" => format!("afade=t=out:st={}:d={}", start, duration),
            "equal_power" => format!("afade=t=in:st={}:d={}:curve=qsin", start, duration),
            "exponential" => format!("afade=t=in:st={}:d={}:curve=exp", start, duration),
            _ => return Err("Tipo de fade desconhecido".into()),
        };

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Returns the duration of the audio file in seconds.
    pub fn get_duration(input: &str) -> Result<f64, String> {
        crate::utils::ffprobe_duration(input)
    }

    /// Returns available fade types with display names.
    pub fn get_fade_types() -> Vec<(&'static str, &'static str)> {
        static STATIC: LazyLock<&[(&str, &str)]> = LazyLock::new(|| {
            &[
                ("in", "Fade In"),
                ("out", "Fade Out"),
                ("equal_power", "Equal Power (suave)"),
                ("exponential", "Exponencial (rápido)"),
            ]
        });
        STATIC.to_vec()
    }
}
