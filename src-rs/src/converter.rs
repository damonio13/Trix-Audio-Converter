//! Audio Conversion Engine
//!
//! Core conversion logic using FFmpeg as the backend. Supports parallel
//! multi-threaded conversion, codec copy mode, audio effects, and
//! progress tracking via atomic counters.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use crate::formats::{OUTPUT_FORMATS, SAMPLE_RATES, CHANNELS, can_copy};
use crate::effects::AudioEffects;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Status of a single file in the conversion queue.
pub struct FileStatus {
    pub filename: String,
    pub input: String,
    pub output: String,
    pub status: String,
    pub progress: u8,
    pub error: String,
    pub elapsed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Aggregated status of an entire conversion batch.
pub struct ConversionStatus {
    pub converting: bool,
    pub total: usize,
    pub converted: usize,
    pub failed: usize,
    pub progress: u8,
    pub elapsed: f64,
    pub eta: f64,
    pub files: HashMap<String, FileStatus>,
}

#[derive(Clone)]
/// Multi-threaded audio converter backed by FFmpeg.
pub struct AudioConverter {
    pub converting: Arc<AtomicBool>,
    pub cpu_count: usize,
    total_files: Arc<AtomicUsize>,
    converted_files: Arc<AtomicUsize>,
    failed_files: Arc<AtomicUsize>,
    /// Uses std::sync::Mutex (not tokio) because locks are held only for
    /// short HashMap operations — never across await points.
    files: Arc<Mutex<HashMap<String, FileStatus>>>,
    file_index: Arc<Mutex<HashMap<String, String>>>,
    batch_start: Arc<Mutex<Option<std::time::Instant>>>,
    cancel_flag: Arc<AtomicBool>,
}

impl AudioConverter {
    /// Creates a new AudioConverter with the specified number of worker threads.
    pub fn new(max_workers: usize) -> Self {
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let _workers = if max_workers > 0 { max_workers } else { cpu_count };

        Self {
            converting: Arc::new(AtomicBool::new(false)),
            cpu_count,
            total_files: Arc::new(AtomicUsize::new(0)),
            converted_files: Arc::new(AtomicUsize::new(0)),
            failed_files: Arc::new(AtomicUsize::new(0)),
            files: Arc::new(Mutex::new(HashMap::new())),
            file_index: Arc::new(Mutex::new(HashMap::new())),
            batch_start: Arc::new(Mutex::new(None)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Starts parallel conversion of all queued files using FFmpeg workers.
    pub fn start(
        &self,
        files: Vec<(String, String, String)>,
        sample_rate: &str,
        channels: &str,
        volume: i32,
        trim_start: f64,
        trim_end: f64,
        output_pattern: &str,
        codec_copy_mode: &str,
        effects: Option<&AudioEffects>,
        output_subfolder: bool,
        max_output_size_mb: f64,
        bit_rate: &str,
    ) -> bool {
        if files.is_empty() {
            return false;
        }

        if self.converting.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_err() {
            return false;
        }
        self.cancel_flag.store(false, Ordering::Relaxed);
        self.failed_files.store(0, Ordering::Relaxed);
        self.converted_files.store(0, Ordering::Relaxed);
        self.total_files.store(files.len(), Ordering::Relaxed);
        if let Ok(mut m) = self.files.lock() { *m = HashMap::new(); }
        if let Ok(mut s) = self.batch_start.lock() { *s = Some(std::time::Instant::now()); }

        // Build audio filters
        let mut afilters = Vec::new();

        let volume = volume.clamp(-20, 20);
        if volume != 0 {
            afilters.push(format!("volume={}dB", volume));
        }
        let trim_start = trim_start.max(0.0);
        let trim_end = if trim_end > 0.0 { trim_end } else { 0.0 };
        if trim_start > 0.0 || trim_end > 0.0 {
            if trim_end > 0.0 {
                afilters.push(format!("atrim={}:{}", trim_start, trim_end));
            } else {
                afilters.push(format!("atrim={}", trim_start));
            }
            afilters.push("asetpts=PTS-STARTPTS".into());
        }

        let has_effects = if let Some(fx) = effects {
            let effect_filters = fx.build_filters();
            if !effect_filters.is_empty() {
                afilters.extend(effect_filters);
                true
            } else {
                false
            }
        } else {
            false
        };

        let sr_value = SAMPLE_RATES.get(sample_rate).copied().unwrap_or("0");
        if sr_value != "0" {
            afilters.push(format!("aresample={}", sr_value));
        }

        let ch_value = CHANNELS.get(channels).copied().unwrap_or("0");
        if ch_value != "0" {
            // Use aformat to set channel count correctly
            afilters.push(format!("aformat=channel_layouts={}c", ch_value));
        }

        // Build final files list
        let mut final_files = Vec::new();
        let mut created_dirs = std::collections::HashSet::new();
        for (i, (inp, out, fmt_key)) in files.iter().enumerate() {
            let mut out_path = std::path::PathBuf::from(out);

            if output_subfolder {
                let subfolder = fmt_key.strip_prefix('.').unwrap_or(fmt_key).split_whitespace().next().unwrap_or("output").to_lowercase();
                out_path = out_path.parent().unwrap_or(Path::new(".")).join(&subfolder).join(out_path.file_name().unwrap_or_default());
                let parent = out_path.parent().unwrap_or(Path::new("."));
                if created_dirs.insert(parent.to_path_buf()) {
                    let _ = std::fs::create_dir_all(parent);
                }
            }

            if !output_pattern.is_empty()
                && !output_pattern.contains('/')
                && !output_pattern.contains('\\')
                && !output_pattern.contains('\0')
            {
                let src = Path::new(inp);
                let name_part = output_pattern
                    .replace("{name}", src.file_stem().unwrap_or_default().to_string_lossy().as_ref())
                    .replace("{n}", &format!("{:03}", i + 1));
                let ext = OUTPUT_FORMATS.get(fmt_key.as_str()).map(|f| f.ext).unwrap_or(".wav");
                out_path = out_path.parent().unwrap_or(Path::new(".")).join(format!("{}{}", name_part, ext));
            }

            final_files.push((inp.clone(), out_path.to_string_lossy().to_string(), fmt_key.clone()));
        }

        // Init file status
        {
            let mut files_map = match self.files.lock() { Ok(m) => m, Err(poisoned) => poisoned.into_inner() };
            let mut index = match self.file_index.lock() { Ok(m) => m, Err(poisoned) => poisoned.into_inner() };
            index.clear();
            for (i, (inp, out, _fmt_key)) in final_files.iter().enumerate() {
                let key = i.to_string();
                files_map.insert(
                    key.clone(),
                    FileStatus {
                        filename: Path::new(inp).file_name().unwrap_or_default().to_string_lossy().to_string(),
                        input: inp.clone(),
                        output: out.clone(),
                        status: "queued".into(),
                        progress: 0,
                        error: String::new(),
                        elapsed: 0.0,
                    },
                );
                index.insert(out.clone(), key);
            }
        }

        // Spawn conversion thread
        let converting = Arc::clone(&self.converting);
        let files = Arc::clone(&self.files);
        let file_index = Arc::clone(&self.file_index);
        let converted = Arc::clone(&self.converted_files);
        let failed = Arc::clone(&self.failed_files);
        let cancel_flag = Arc::clone(&self.cancel_flag);
        let codec_copy_mode = codec_copy_mode.to_string();
        let afilters = Arc::new(afilters);

        let sample_rate = sample_rate.to_string();
        let channels = channels.to_string();
        let bit_rate = bit_rate.to_string();

        thread::spawn(move || {
            let total = final_files.len();
            let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
            // Divide files evenly across workers using ceiling division:
            // chunk_size = ceil(total / workers) ensures no worker gets more files than others.
            let chunk_size = (total + workers - 1) / workers;

            let mut handles = Vec::new();

            for chunk in final_files.chunks(chunk_size) {
                let chunk = chunk.to_vec();
                let afilters = Arc::clone(&afilters);
                let cancel = Arc::clone(&cancel_flag);
                let files = Arc::clone(&files);
                let file_index = Arc::clone(&file_index);
                let converted = Arc::clone(&converted);
                let failed = Arc::clone(&failed);
                let ccm = codec_copy_mode.clone();
                let sr = sample_rate.clone();
                let ch = channels.clone();
                let br = bit_rate.clone();

                let handle = thread::spawn(move || {
                    for (_i, (inp, out, fmt_key)) in chunk.iter().enumerate() {
                        if cancel.load(Ordering::Relaxed) {
                            return;
                        }

                        let fmt = match OUTPUT_FORMATS.get(fmt_key.as_str()) {
                            Some(f) => f,
                            None => continue,
                        };
                        let codec_args = fmt.args;
                        let output_ext = fmt.ext;

                        Self::convert_one(
                            inp, out, codec_args, &afilters,
                            &ccm, output_ext,
                            has_effects, max_output_size_mb, &files, &file_index, &converted, &failed,
                            &sr, &ch, &br,
                        );
                    }
                });

                handles.push(handle);
            }

            for h in handles {
                let _ = h.join();
            }

            converting.store(false, Ordering::Relaxed);
        });

        true
    }

    /// Cancels all in-progress conversions.
    pub fn cancel(&self) {
        if self.converting.load(Ordering::SeqCst) {
            self.cancel_flag.store(true, Ordering::SeqCst);
        }
    }

    /// Returns the current conversion status including progress and per-file results.
    pub fn get_status(&self) -> ConversionStatus {
        let converting = self.converting.load(Ordering::Relaxed);
        let total = self.total_files.load(Ordering::Relaxed);
        let converted = self.converted_files.load(Ordering::Relaxed);
        let failed = self.failed_files.load(Ordering::Relaxed);

        let progress = if total > 0 {
            let pct = converted as f64 / total as f64 * 100.0;
            (pct.min(100.0).max(0.0)) as u8
        } else {
            0
        };

        let (elapsed, eta) = if let Some(start) = *self.batch_start.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) {
            let e = start.elapsed().as_secs_f64();
            let eta = if converted > 0 && total > 0 && e > 0.0 && converted <= total {
                let per_file = e / converted as f64;
                total.saturating_sub(converted) as f64 * per_file
            } else {
                0.0
            };
            (e, eta)
        } else {
            (0.0, 0.0)
        };

        let files_snapshot = {
            let guard = self.files.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.clone()
        };

        let status = ConversionStatus {
            converting,
            total,
            converted,
            failed,
            progress,
            elapsed: elapsed.round(),
            eta: eta.round(),
            files: files_snapshot,
        };

        status
    }

    /// Updates the status of a single file in the shared status map using its output path as key.
    /// Looks up the numeric key via the `file_index` reverse map to avoid storing paths twice.
    fn update_file_status(
        files: &Mutex<HashMap<String, FileStatus>>,
        file_index: &Mutex<HashMap<String, String>>,
        output_path: &str,
        status: &str,
        error: &str,
        elapsed: f64,
        progress: u8,
    ) {
        let key = {
            let idx = match file_index.lock() { Ok(m) => m, Err(poisoned) => poisoned.into_inner() };
            idx.get(output_path).cloned()
        };
        if let Some(key) = key {
            let mut map = match files.lock() { Ok(m) => m, Err(poisoned) => poisoned.into_inner() };
            if let Some(s) = map.get_mut(&key) {
                s.status = status.into();
                s.error = error.into();
                s.elapsed = elapsed;
                s.progress = progress;
            }
        }
    }

    /// Returns `true` if the input→output conversion can use FFmpeg's stream-copy mode
    /// (skipping re-encoding entirely). Copy is blocked when audio filters or effects are active.
    fn should_copy(
        codec_copy_mode: &str,
        input_ext: &str,
        output_ext: &str,
        afilters: &[String],
        has_effects: bool,
    ) -> bool {
        if !afilters.is_empty() || has_effects {
            return false;
        }
        match codec_copy_mode {
            "force" => true,
            "auto" => can_copy(input_ext, output_ext),
            _ => false,
        }
    }

    /// Builds the `ffmpeg` [`Command`] for a single file conversion.
    ///
    /// Uses `-c:a copy` when `do_copy` is `true`, otherwise applies `afilters` and `codec_args`.
    /// The output path is separated from options with `--` to prevent path injection.
    fn build_ffmpeg_cmd(
        input_path: &str,
        output_path: &str,
        codec_args: &[&str],
        afilters: &[String],
        do_copy: bool,
    ) -> Command {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-nostats", "-i", input_path]);
        if do_copy {
            cmd.args(["-c:a", "copy"]);
        } else {
            if !afilters.is_empty() {
                cmd.args(["-af", &afilters.join(",")]);
            }
            cmd.args(codec_args);
        }
        cmd.args(["--", output_path]);
        cmd
    }

    /// Converts a single file by spawning an `ffmpeg` child process.
    ///
    /// - Overrides `-b:a`, `-ar`, and `-ac` codec args with user-specified values when non-zero.
    /// - Deletes the output if it exceeds `max_output_size_mb`.
    /// - Updates the shared status map throughout: `"processing"` → `"completed"` or `"error"`.
    fn convert_one(
        input_path: &str,
        output_path: &str,
        codec_args: &[&str],
        afilters: &[String],
        codec_copy_mode: &str,
        output_ext: &str,
        has_effects: bool,
        max_output_size_mb: f64,
        files: &Mutex<HashMap<String, FileStatus>>,
        file_index: &Mutex<HashMap<String, String>>,
        converted: &AtomicUsize,
        failed: &AtomicUsize,
        sample_rate: &str,
        channels: &str,
        bit_rate: &str,
    ) {
        Self::update_file_status(files, file_index, output_path, "processing", "", 0.0, 0);

        let input_ext = {
            let ext = Path::new(input_path).extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let mut buf = [0u8; 64];
            buf[0] = b'.';
            let bytes = ext.as_bytes();
            let n = bytes.len().min(buf.len() - 1);
            buf[1..=n].copy_from_slice(&bytes[..n]);
            String::from_utf8_lossy(&buf[..=n]).to_string()
        };

        let sr_value = SAMPLE_RATES.get(sample_rate).copied().unwrap_or("0");
        let ch_value = CHANNELS.get(channels).copied().unwrap_or("0");

        let mut final_codec_args = Vec::new();
        let mut idx = 0;
        while idx < codec_args.len() {
            if codec_args[idx] == "-b:a" && !bit_rate.is_empty() && bit_rate != "Original" {
                final_codec_args.push("-b:a".to_string());
                final_codec_args.push(bit_rate.to_string());
                idx += 2;
            } else if codec_args[idx] == "-ar" && sr_value != "0" {
                final_codec_args.push("-ar".to_string());
                final_codec_args.push(sr_value.to_string());
                idx += 2;
            } else if codec_args[idx] == "-ac" && ch_value != "0" {
                final_codec_args.push("-ac".to_string());
                final_codec_args.push(ch_value.to_string());
                idx += 2;
            } else {
                final_codec_args.push(codec_args[idx].to_string());
                idx += 1;
            }
        }

        let final_codec_args_ref: Vec<&str> = final_codec_args.iter().map(|s| s.as_str()).collect();

        let do_copy = Self::should_copy(codec_copy_mode, &input_ext, output_ext, afilters, has_effects);
        let mut cmd = Self::build_ffmpeg_cmd(input_path, output_path, &final_codec_args_ref, afilters, do_copy);

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let start = std::time::Instant::now();

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => {
                Self::update_file_status(files, file_index, output_path, "error", "Falha ao iniciar ffmpeg", 0.0, 0);
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let result = child.wait_with_output();
        let elapsed = start.elapsed().as_secs_f64();

        match result {
            Ok(output) => {
                if output.status.success() {
                    if max_output_size_mb > 0.0 {
                        if let Ok(meta) = std::fs::metadata(output_path) {
                            let out_size_mb = meta.len() as f64 / (1024.0 * 1024.0);
                            if out_size_mb > max_output_size_mb {
                                let _ = std::fs::remove_file(output_path);
                                let err = format!("Tamanho {:.1}MB excede limite de {:.1}MB", out_size_mb, max_output_size_mb);
                                Self::update_file_status(files, file_index, output_path, "error", &err, 0.0, 0);
                                failed.fetch_add(1, Ordering::Relaxed);
                                return;
                            }
                        }
                    }
                    Self::update_file_status(files, file_index, output_path, "completed", "", elapsed, 100);
                    converted.fetch_add(1, Ordering::Relaxed);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    println!("FFMPEG ERROR OCCURRED:\n{}", stderr);
                    // Truncate FFmpeg stderr to 200 chars to prevent UI overflow from verbose output.
                    let err_msg: String = stderr.chars()
                        .filter(|c| !c.is_control() || *c == '\n')
                        .take(200)
                        .collect();
                    Self::update_file_status(files, file_index, output_path, "error", &err_msg, 0.0, 0);
                    failed.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(_) => {
                Self::update_file_status(files, file_index, output_path, "error", "Erro interno de conversao", 0.0, 0);
                failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl Drop for AudioConverter {
    /// Signals any active conversion to cancel when the converter is dropped,
    /// preventing orphaned FFmpeg processes from outliving their parent.
    fn drop(&mut self) {
        if self.converting.load(Ordering::SeqCst) {
            self.cancel_flag.store(true, Ordering::SeqCst);
        }
    }
}
