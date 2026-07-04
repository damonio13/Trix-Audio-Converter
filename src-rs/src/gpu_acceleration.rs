//! GPU acceleration detection (NVENC, QSV, VAAPI).
//!
//! Probes the system for available hardware encoders and reports
//! supported GPU-accelerated codecs for faster audio/video conversion.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, OnceLock};

/// System GPU information with available hardware encoders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub has_nvenc: bool,
    pub has_qsv: bool,
    pub has_videotoolbox: bool,
    pub has_vaapi: bool,
    pub nvenc_version: Option<String>,
    pub qsv_version: Option<String>,
    pub devices: Vec<GpuDevice>,
}

/// A detected GPU hardware encoder device with capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub name: String,
    pub encoder: String,
    pub max_width: u32,
    pub max_height: u32,
    pub pixel_formats: Vec<String>,
}

fn ffmpeg_encoder_list() -> &'static str {
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED.get_or_init(|| {
        std::process::Command::new("ffmpeg")
            .args(["-hide_banner", "-encoders"])
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
            .unwrap_or_default()
    })
}

fn check_encoder_available(encoder_name: &str) -> bool {
    ffmpeg_encoder_list().contains(encoder_name)
}

/// Detects and reports available GPU-accelerated video encoders.
pub struct GpuAcceleration;

impl GpuAcceleration {
    /// Detects all available GPU hardware encoders on the system.
    pub fn detect() -> GpuInfo {
        let mut info = GpuInfo {
            has_nvenc: false,
            has_qsv: false,
            has_videotoolbox: false,
            has_vaapi: false,
            nvenc_version: None,
            qsv_version: None,
            devices: Vec::new(),
        };

        if check_encoder_available("h264_nvenc") {
            info.has_nvenc = true;
            info.nvenc_version = Some("Detected".into());
            info.devices.push(GpuDevice {
                name: "NVIDIA NVENC".into(),
                encoder: "h264_nvenc".into(),
                max_width: 4096,
                max_height: 2160,
                pixel_formats: vec!["yuv420p".into(), "yuv444p".into()],
            });
            if check_encoder_available("hevc_nvenc") {
                if let Some(d) = info.devices.iter_mut().find(|d| d.encoder == "h264_nvenc") {
                    d.encoder = "h264_nvenc/hevc_nvenc".into();
                }
            }
        }

        if check_encoder_available("h264_qsv") {
            info.has_qsv = true;
            info.qsv_version = Some("Detected".into());
            info.devices.push(GpuDevice {
                name: "Intel Quick Sync".into(),
                encoder: "h264_qsv".into(),
                max_width: 4096,
                max_height: 2160,
                pixel_formats: vec!["nv12".into(), "p010".into()],
            });
        }

        if check_encoder_available("h264_vaapi") {
            info.has_vaapi = true;
            info.devices.push(GpuDevice {
                name: "VAAPI".into(),
                encoder: "h264_vaapi".into(),
                max_width: 4096,
                max_height: 2160,
                pixel_formats: vec!["vaapi".into()],
            });
        }

        if check_encoder_available("h264_videotoolbox") {
            info.has_videotoolbox = true;
            info.devices.push(GpuDevice {
                name: "Apple VideoToolbox".into(),
                encoder: "h264_videotoolbox".into(),
                max_width: 4096,
                max_height: 2160,
                pixel_formats: vec!["yuv420p".into()],
            });
        }

        info
    }

    /// Returns the best available hardware encoder, or None if unavailable.
    pub fn get_best_encoder() -> Option<String> {
        let info = Self::detect();
        if info.has_nvenc {
            Some("h264_nvenc".into())
        } else if info.has_qsv {
            Some("h264_qsv".into())
        } else if info.has_videotoolbox {
            Some("h264_videotoolbox".into())
        } else if info.has_vaapi {
            Some("h264_vaapi".into())
        } else {
            None
        }
    }

    /// Returns all supported GPU-accelerated codecs with encoder and codec names.
    pub fn get_gpu_accelerated_codecs() -> &'static [(&'static str, &'static str, &'static str)] {
        static STATIC: LazyLock<&[(&str, &str, &str)]> = LazyLock::new(|| {
            &[
                ("h264_nvenc", "NVIDIA NVENC", "H.264"),
                ("hevc_nvenc", "NVIDIA NVENC", "H.265/HEVC"),
                ("h264_qsv", "Intel Quick Sync", "H.264"),
                ("hevc_qsv", "Intel Quick Sync", "H.265/HEVC"),
                ("h264_vaapi", "VAAPI (Linux)", "H.264"),
                ("h264_videotoolbox", "Apple VideoToolbox", "H.264"),
            ]
        });
        &*STATIC
    }
}
