//! Audio file duplicate detection using fingerprinting.
//!
//! Computes acoustic fingerprints of audio files and identifies
//! near-identical duplicates even across different formats or bitrates.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::collections::HashMap;

/// Acoustic fingerprint of an audio file for duplicate detection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioFingerprint {
    pub path: String,
    pub duration: f64,
    pub sample_rate: u64,
    pub channels: u32,
    pub codec: String,
    pub bitrate: u64,
    pub size: u64,
}

/// A group of duplicate audio files with size savings information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateGroup {
    pub files: Vec<String>,
    pub total_size: u64,
    pub can_save: u64,
}

/// Detects duplicate audio files using acoustic fingerprinting.
pub struct DuplicateDetector;

impl DuplicateDetector {
    /// Generates an acoustic fingerprint for the given audio file.
    pub fn get_fingerprint(path: &str) -> Result<AudioFingerprint, String> {
        let data = crate::utils::ffprobe_json(path)?;

        let duration = data["format"]["duration"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let bitrate = data["format"]["bit_rate"].as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let streams = match data.get("streams").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Err("No streams found".into()),
        };
        let audio_stream = streams.iter()
            .find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("audio"));

        let codec = audio_stream
            .and_then(|s| s.get("codec_name").and_then(|v| v.as_str()))
            .unwrap_or("unknown")
            .to_string();

        let sample_rate = audio_stream
            .and_then(|s| s.get("sample_rate").and_then(|v| v.as_str()))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let channels = audio_stream
            .and_then(|s| s.get("channels").and_then(|v| v.as_str()))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(AudioFingerprint {
            path: path.to_string(),
            duration,
            sample_rate,
            channels,
            codec,
            bitrate,
            size,
        })
    }

    fn group_and_find(
        paths: &[String],
        key_fn: impl Fn(&AudioFingerprint) -> String,
    ) -> Vec<DuplicateGroup> {
        let mut fingerprints: Vec<AudioFingerprint> = Vec::new();
        for path in paths {
            if let Ok(fp) = Self::get_fingerprint(path) {
                fingerprints.push(fp);
            }
        }
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, fp) in fingerprints.iter().enumerate() {
            let key = key_fn(fp);
            groups.entry(key).or_default().push(i);
        }
        Self::build_groups(groups, &fingerprints)
    }

    /// Finds approximate duplicates by duration grouping.
    pub fn find_duplicates(paths: &[String]) -> Vec<DuplicateGroup> {
        Self::group_and_find(paths, |fp| format!("{}", (fp.duration * 10.0) as u64))
    }

    /// Finds exact duplicates by codec, duration, and sample rate.
    pub fn find_exact_duplicates(paths: &[String]) -> Vec<DuplicateGroup> {
        Self::group_and_find(paths, |fp| {
            format!("{}_{:.3}_{}", fp.codec, fp.duration, fp.sample_rate)
        })
    }

    fn build_groups(groups: HashMap<String, Vec<usize>>, fingerprints: &[AudioFingerprint]) -> Vec<DuplicateGroup> {
        let mut duplicates = Vec::new();
        for (_, indices) in groups {
            if indices.len() > 1 {
                let total_size: u64 = indices.iter().map(|&i| fingerprints[i].size).sum();
                let min_size = indices.iter().map(|&i| fingerprints[i].size).min().unwrap_or(0);
                let can_save = total_size - min_size;

                duplicates.push(DuplicateGroup {
                    files: indices.iter().map(|&i| fingerprints[i].path.clone()).collect(),
                    total_size,
                    can_save,
                });
            }
        }

        duplicates.sort_by(|a, b| b.can_save.cmp(&a.can_save));
        duplicates
    }
}
