//! CUE sheet parsing for track splitting.
//!
//! Parses CUE sheet files to extract track boundaries, titles, and
//! performer information for accurate audio file segmentation.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// A single track parsed from a CUE sheet.
pub struct CueTrack {
    pub number: u32,
    pub title: String,
    pub artist: String,
    pub start_time: f64,
    pub end_time: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Parsed CUE sheet containing track boundaries and metadata.
pub struct CueSheet {
    pub file: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: String,
    pub tracks: Vec<CueTrack>,
}

impl CueSheet {
    /// Parses a CUE sheet file into a CueSheet structure.
    pub fn parse(cue_path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(cue_path)
            .map_err(|e| format!("Failed to read CUE file: {}", e))?;

        let mut file = String::new();
        let mut artist = String::new();
        let mut album = String::new();
        let mut genre = String::new();
        let mut year = String::new();
        let mut tracks: Vec<CueTrack> = Vec::new();
        let mut current_track: Option<CueTrack> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("FILE") {
                if let Some(pos) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        if pos != end {
                            file = line[pos + 1..end].to_string();
                        }
                    }
                }
            } else if line.starts_with("PERFORMER") {
                let val = Self::extract_quoted(line);
                if current_track.is_some() {
                    if let Some(ref mut t) = current_track {
                        t.artist = val;
                    }
                } else {
                    artist = val;
                }
            } else if line.starts_with("TITLE") {
                let val = Self::extract_quoted(line);
                if current_track.is_some() {
                    if let Some(ref mut t) = current_track {
                        t.title = val;
                    }
                } else {
                    album = val;
                }
            } else if line.starts_with("GENRE") {
                genre = Self::extract_quoted(line);
            } else if line.starts_with("DATE") {
                year = Self::extract_quoted(line);
            } else if line.starts_with("TRACK") {
                if let Some(ref mut t) = current_track {
                    tracks.push(t.clone());
                }
                let num = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                current_track = Some(CueTrack {
                    number: num,
                    title: String::new(),
                    artist: artist.clone(),
                    start_time: 0.0,
                    end_time: None,
                });
            } else if line.starts_with("INDEX 01") {
                if let Some(ref mut t) = current_track {
                    let time = Self::parse_time(line);
                    t.start_time = time;
                }
            }
        }

        if let Some(ref mut t) = current_track {
            tracks.push(t.clone());
        }

        // Calculate end times
        for i in 0..tracks.len() {
            if i + 1 < tracks.len() {
                tracks[i].end_time = Some(tracks[i + 1].start_time);
            }
        }

        Ok(CueSheet {
            file,
            artist,
            album,
            genre,
            year,
            tracks,
        })
    }

    fn extract_quoted(line: &str) -> String {
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if start != end {
                    return line[start + 1..end].to_string();
                }
            }
        }
        line.split_whitespace().skip(1).collect::<Vec<&str>>().join(" ")
    }

    fn parse_time(line: &str) -> f64 {
        let time_str = line.split_whitespace().last().unwrap_or("0:00:00");
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 3 {
            let min: f64 = parts[0].parse().unwrap_or(0.0);
            let sec: f64 = parts[1].parse().unwrap_or(0.0);
            let frame: f64 = parts[2].parse().unwrap_or(0.0);
            min * 60.0 + sec + frame / 75.0
        } else if parts.len() == 2 {
            let min: f64 = parts[0].parse().unwrap_or(0.0);
            let sec: f64 = parts[1].parse().unwrap_or(0.0);
            min * 60.0 + sec
        } else {
            0.0
        }
    }

    /// Splits an audio file into individual tracks using the CUE sheet timings.
    pub fn split_audio(&self, audio_path: &str, output_dir: &str) -> Result<Vec<String>, String> {
        let mut outputs = Vec::new();

        for track in &self.tracks {
            let ext = Path::new(audio_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("flac");

            let output_file = format!("{}/track{:02}.{}", output_dir, track.number, ext);

            let start = crate::utils::format_mm_ss_ms(track.start_time, 2);
            let dur = track.end_time.map(|end| crate::utils::format_mm_ss_ms(end - track.start_time, 2));

            let mut args = vec!["-i", audio_path, "-ss", &start];

            if let Some(ref d) = dur {
                args.extend(["-t", d]);
            }

            args.extend(["-c", "copy", "-y", "--", &output_file]);

            let out = crate::utils::run_ffmpeg_raw(&args)
                .map_err(|e| format!("ffmpeg failed: {}", e))?;

            if out.status.success() {
                outputs.push(output_file);
            }
        }

        Ok(outputs)
    }
}
