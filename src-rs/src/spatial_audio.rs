//! Spatial audio processing and 3D audio effects
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Spatial audio processing and 3D audio effects.
pub struct SpatialAudio;

impl SpatialAudio {
    /// Converts audio to binaural 3D headphone output.
    pub fn to_binaural(
        input: &str,
        output: &str,
        room_size: f64,
        damping: f64,
    ) -> Result<String, String> {
        let af = format!(
            "stereotools=mlev=1.0:slev=1.0,freeverb=roomsize={}:damp={}:wet=0.3:dry=0.7:width=0.9",
            room_size, damping
        );
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Upmixes stereo audio to 5.1 surround sound.
    pub fn to_surround_51(input: &str, output: &str) -> Result<String, String> {
        crate::utils::run_ffmpeg_af(input, output, crate::utils::pan_filter("stereo_to_surround_51"))
    }

    /// Converts audio to ambisonic format of the given order.
    pub fn to_ambisonic(input: &str, output: &str, order: u32) -> Result<String, String> {
        let channels = (order + 1).pow(2);
        let af = format!("pan={}|c0=c0|c1=c1", channels);
        let channels_str = channels.to_string();

        crate::utils::run_ffmpeg_raw_checked(&[
            "-hide_banner",
            "-i", input,
            "-af", &af,
            "-ac", &channels_str,
            "-y", "--", output,
        ])?;

        Ok("Sucesso".into())
    }

    /// Downmixes surround audio to stereo.
    pub fn downmix_surround(input: &str, output: &str) -> Result<String, String> {
        crate::utils::run_ffmpeg_af(input, output, crate::utils::pan_filter("surround_51_to_stereo"))
    }

    /// Returns the list of available spatial audio modes.
    pub fn get_spatial_modes() -> Vec<(&'static str, &'static str)> {
        static SPATIAL_MODES: LazyLock<&[(&str, &str)]> = LazyLock::new(|| &[
            ("binaural", "Binaural (3D headphone)"),
            ("surround_51", "5.1 Surround"),
            ("surround_71", "7.1 Surround"),
            ("ambisonic_foa", "Ambisonic 1ª Ordem (4ch)"),
            ("ambisonic_soa", "Ambisonic 2ª Ordem (9ch)"),
            ("stereo_upmix", "Upmix Estéreo → Surround"),
        ]);
        SPATIAL_MODES.to_vec()
    }

    /// Returns available room reverb presets as (name, room_size, reverberance) tuples.
    pub fn get_room_presets() -> Vec<(&'static str, f64, f64)> {
        static ROOM_PRESETS: LazyLock<&[(&str, f64, f64)]> = LazyLock::new(|| &[
            ("studio", 0.2, 0.8),
            ("small_room", 0.4, 0.6),
            ("large_room", 0.6, 0.4),
            ("hall", 0.8, 0.3),
            ("cathedral", 0.95, 0.2),
        ]);
        ROOM_PRESETS.to_vec()
    }
}
