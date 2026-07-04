//! Audio channel mixing: stereo to mono, surround downmix, etc.
//!
//! Converts between different channel configurations including stereo,
//! mono, 5.1 surround, and custom channel mappings with balance control.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Mixes and converts between different audio channel configurations.
pub struct ChannelMixer;

impl ChannelMixer {
    /// Converts audio between channel modes (stereo, mono, surround, etc.).
    pub fn mix_channels(
        input: &str,
        output: &str,
        mode: &str,
    ) -> Result<String, String> {
        let af = match mode {
            "stereo_to_mono" => crate::utils::pan_filter("mono"),
            "mono_to_stereo" => "pan=stereo|c0=c0|c1=c0",
            "stereo_to_left" => crate::utils::pan_filter("left"),
            "stereo_to_right" => crate::utils::pan_filter("right"),
            "surround_51_to_stereo" => crate::utils::pan_filter("surround_51_to_stereo"),
            "stereo_to_surround_51" => crate::utils::pan_filter("stereo_to_surround_51"),
            "stereo_to_binaural" => "stereotools=mlev=1.0:slev=1.0",
            _ => return Err("Modo desconhecido".into()),
        };

        crate::utils::run_ffmpeg_af(input, output, af)
    }

    /// Returns the list of available channel mixing modes.
    pub fn get_modes() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("stereo_to_mono", "Estéreo → Mono"),
            ("mono_to_stereo", "Mono → Estéreo"),
            ("stereo_to_left", "Estéreo → Canal Esquerdo"),
            ("stereo_to_right", "Estéreo → Canal Direito"),
            ("surround_51_to_stereo", "5.1 Surround → Estéreo"),
            ("stereo_to_surround_51", "Estéreo → 5.1 Surround"),
            ("stereo_to_binaural", "Estéreo → Binaural (3D)"),
        ]);
        &*STATIC
    }
}
