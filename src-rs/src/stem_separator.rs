//! Audio stem separation (vocals, drums, bass, other)
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Separates audio into individual stems (vocals, drums, bass, etc.).
pub struct StemSeparator;

impl StemSeparator {
    /// Separates audio into the requested stems using FFmpeg filters.
    pub fn separate(
        input: &str,
        output_dir: &str,
        stems: &[&str],
    ) -> Result<Vec<String>, String> {
        crate::utils::validate_input_output(input, output_dir)?;

        let mut outputs = Vec::new();

        for stem in stems {
            let output = match *stem {
                "vocals" => format!("{}/vocals.wav", output_dir),
                "drums" => format!("{}/drums.wav", output_dir),
                "bass" => format!("{}/bass.wav", output_dir),
                "other" => format!("{}/other.wav", output_dir),
                "piano" => format!("{}/piano.wav", output_dir),
                "guitar" => format!("{}/guitar.wav", output_dir),
                _ => format!("{}/{}.wav", output_dir, stem),
            };

            let af = match *stem {
                "vocals" => "pan=mono|c0=c0-c1",
                "drums" => "pan=mono|c0=c0+c1,lowpass=f=500:poles=2,highpass=f=50:poles=2",
                "bass" => "lowpass=f=200:poles=2",
                "other" => "pan=mono|c0=c0+c1,bandpass=f=1000:width_type=h:w=500",
                _ => "",
            };

            if af.is_empty() {
                continue;
            }

            if crate::utils::run_ffmpeg_af(input, &output, af).is_ok() {
                outputs.push(output);
            }
        }

        Ok(outputs)
    }

    /// Separates audio into stems using Spleeter (external tool).
    pub fn separate_with_spleeter(
        input: &str,
        output_dir: &str,
        model: &str,
    ) -> Result<Vec<String>, String> {
        crate::utils::validate_input_output(input, output_dir)?;

        let stems_arg = match model {
            "2stems" => "vocals,accompaniment",
            "4stems" => "vocals,drums,bass,other",
            "5stems" => "vocals,drums,bass,piano,other",
            _ => "vocals,drums,bass,other",
        };

        let out = std::process::Command::new("spleeter")
            .args([
                "separate",
                "--", "-i", input,
                "--", "-o", output_dir,
                "-p", model,
                "-f", "{instrument}.{codec}",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if out.status.success() {
            let mut results = Vec::new();
            for stem in stems_arg.split(',') {
                let path = format!("{}/{}.wav", output_dir, stem);
                if std::path::Path::new(&path).exists() {
                    results.push(path);
                }
            }
            Ok(results)
        } else {
            Err("Spleeter não disponível. Use o método FFT.".into())
        }
    }

    /// Returns available stem-separation models as (id, display_name, quality_score) tuples.
    pub fn get_models() -> Vec<(&'static str, &'static str, u32)> {
        static MODELS: LazyLock<&[(&str, &str, u32)]> = LazyLock::new(|| &[
            ("2stems", "Vocals + Accompaniment", 2),
            ("4stems", "Vocals + Drums + Bass + Other", 4),
            ("5stems", "Vocals + Drums + Bass + Piano + Other", 5),
        ]);
        MODELS.to_vec()
    }

    /// Returns the stem types produced by separation (e.g. vocals, drums, bass, other).
    pub fn get_stems() -> Vec<(&'static str, &'static str)> {
        static STEMS: LazyLock<&[(&str, &str)]> = LazyLock::new(|| &[
            ("vocals", "Vocals"),
            ("drums", "Bateria"),
            ("bass", "Baixo"),
            ("other", "Outros"),
            ("piano", "Piano"),
            ("guitar", "Guitarra"),
        ]);
        STEMS.to_vec()
    }
}
