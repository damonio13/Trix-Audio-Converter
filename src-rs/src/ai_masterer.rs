//! AI-powered audio mastering with automatic EQ, compression, and limiting.
//!
//! Applies intelligent audio processing chains to normalize and enhance
//! audio files using machine learning models.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// AI-powered audio mastering with genre-aware EQ, compression, and limiting.
pub struct AiMasterer;

impl AiMasterer {
    /// Automatically masters an audio file with genre-specific EQ, compression, and loudness normalization.
    pub fn auto_master(
        input: &str,
        output: &str,
        genre: &str,
        intensity: &str,
        target_lufs: f64,
    ) -> Result<String, String> {
        let (eq_boost, comp_ratio, limiter) = match genre {
            "rock" => (vec![("bass", 3.0), ("treble", 2.0)], 4.0, -1.0),
            "pop" => (vec![("bass", 2.0), ("treble", 3.0)], 3.0, -1.0),
            "jazz" => (vec![("bass", 1.0), ("treble", 1.0)], 2.0, -1.5),
            "classical" => (vec![("bass", 0.5), ("treble", 0.5)], 1.5, -2.0),
            "edm" => (vec![("bass", 5.0), ("treble", 3.0)], 5.0, -0.5),
            "hiphop" => (vec![("bass", 4.0), ("treble", 1.0)], 4.0, -1.0),
            "acoustic" => (vec![("bass", 1.0), ("treble", 2.0)], 2.0, -1.5),
            "voice" => (vec![("bass", 0.0), ("treble", 2.0)], 3.0, -1.0),
            _ => (vec![("bass", 2.0), ("treble", 2.0)], 3.0, -1.0),
        };

        let intensity_mult = match intensity {
            "light" => 0.5,
            "medium" => 1.0,
            "heavy" => 1.5,
            _ => 1.0,
        };

        let mut filters = Vec::new();

        for (freq_type, gain) in &eq_boost {
            let adjusted_gain = gain * intensity_mult;
            let filter = match *freq_type {
                "bass" => format!("bass=g={:.1}:f=100", adjusted_gain),
                "treble" => format!("treble=g={:.1}:f=3000", adjusted_gain),
                _ => continue,
            };
            filters.push(filter);
        }

        let adjusted_ratio = comp_ratio * intensity_mult;
        filters.push(format!(
            "compand=attacks=0.3:decays=0.8:points=-80/-80|-45/-45|-27/-20|0/-7|20/-7:gain=0:volume=-90:delay=0.15:ratio={:.1}",
            adjusted_ratio
        ));

        filters.push(format!("loudnorm=I={}:TP={}:LRA=11", target_lufs, limiter));

        let af = filters.join(",");
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Returns the list of supported music genres for mastering presets.
    pub fn get_genres() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("rock", "Rock"),
            ("pop", "Pop"),
            ("jazz", "Jazz"),
            ("classical", "Clássica"),
            ("edm", "EDM / Eletrônica"),
            ("hiphop", "Hip-Hop / Rap"),
            ("acoustic", "Acústica"),
            ("voice", "Voz / Podcast"),
            ("metal", "Metal"),
            ("country", "Country"),
            ("reggae", "Reggae"),
            ("blues", "Blues"),
            ("latin", "Latina"),
            ("rnb", "R&B / Soul"),
        ]);
        &*STATIC
    }

    /// Returns the available mastering intensity levels.
    pub fn get_intensities() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("light", "Leve (sutil)"),
            ("medium", "Médio (balanceado)"),
            ("heavy", "Forte (agressivo)"),
        ]);
        &*STATIC
    }

    /// Matches the loudness and tonal balance of a reference track.
    pub fn match_reference(
        input: &str,
        reference: &str,
        output: &str,
    ) -> Result<String, String> {
        let input_stats = Self::get_stats(input)?;
        let ref_stats = Self::get_stats(reference)?;

        let gain_diff = ref_stats.0 - input_stats.0;
        let volume = 10.0_f64.powf(gain_diff / 20.0);

        let af = format!("volume={:.4},loudnorm=I=-14:TP=-1.0:LRA=11", volume);
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    fn get_stats(input: &str) -> Result<(f64, f64), String> {
        let stderr = crate::utils::run_ffmpeg_af_stderr(
            input,
            "loudnorm=I=-16:TP=-1.5:print_format=json",
        )?;

        let parsed = crate::utils::parse_loudnorm_json(&stderr)?;
        Ok((parsed.input_i, parsed.input_tp))
    }
}
