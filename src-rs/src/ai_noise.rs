//! AI-based noise reduction and audio cleanup.
//!
//! Uses neural network models to identify and remove background noise,
//! hum, hiss, and other unwanted artifacts from audio recordings.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// AI-based noise reduction using FFT, RNN, and non-local means algorithms.
pub struct AiNoiseReducer;

impl AiNoiseReducer {
    /// Reduces noise in a single audio file using the specified method and sensitivity.
    pub fn reduce_noise(
        input: &str,
        output: &str,
        sensitivity: f64,
        method: &str,
    ) -> Result<String, String> {
        let af = match method {
            "afftdn" => {
                let nr = (sensitivity * 100.0) as i32;
                format!("afftdn=nf=-25:tn=1:nt=w:om=o:{}", nr)
            }
            "arnndn" => "arnndn=model=cb.rnnn".to_string(),
            "highpass_lowpass" => {
                let hp = 80.0;
                let lp = 8000.0;
                format!("highpass=f={}:poles=2,lowpass=f={}:poles=2", hp, lp)
            }
            "anlmdn" => {
                let s = sensitivity * 10.0;
                format!("anlmdn=s={}:p=0.002:m=0.002:r=0.002", s)
            }
            "agate" => {
                let threshold = -40.0 + (sensitivity * 20.0);
                format!("agate=threshold={}:ratio=2:attack=5:release=50", threshold)
            }
            _ => format!("afftdn=nf=-25:tn=1"),
        };

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Reduces noise in multiple audio files, writing denoised outputs to a directory.
    pub fn reduce_noise_batch(
        inputs: &[String],
        output_dir: &str,
        sensitivity: f64,
        method: &str,
    ) -> Result<Vec<String>, String> {
        let mut results = Vec::new();

        for input in inputs {
            let filename = std::path::Path::new(input)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let ext = std::path::Path::new(input)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("wav");
            let output = format!("{}/{}_denoised.{}", output_dir, filename, ext);

            match Self::reduce_noise(input, &output, sensitivity, method) {
                Ok(_) => results.push(output),
                Err(e) => eprintln!("Erro em {}: {}", input, e),
            }
        }

        Ok(results)
    }

    /// Returns the list of available noise reduction methods.
    pub fn get_methods() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("afftdn", "FFT Denoiser (recomendado)"),
            ("arnndn", "RNN Denoiser (AI neural)"),
            ("anlmdn", "Non-Local Means"),
            ("highpass_lowpass", "Highpass + Lowpass"),
            ("agate", "Noise Gate"),
        ]);
        &*STATIC
    }

    /// Returns preset noise reduction configurations (label, sensitivity, method).
    pub fn get_presets() -> &'static [(&'static str, f64, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, f64, &'static str)]> = LazyLock::new(|| &[
            ("Leve", 0.3, "afftdn"),
            ("Médio", 0.5, "afftdn"),
            ("Forte", 0.8, "afftdn"),
            ("Voz (Podcast)", 0.6, "afftdn"),
            ("Música", 0.4, "afftdn"),
            ("Ambiente", 0.7, "anlmdn"),
            ("Neural (AI)", 0.5, "arnndn"),
        ]);
        &*STATIC
    }
}
