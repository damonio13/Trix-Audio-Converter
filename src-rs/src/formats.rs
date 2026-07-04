//! Audio Format Definitions
//!
//! Defines all 106 supported output formats with their FFmpeg arguments,
//! file extensions, categories, and sample rate/channel configurations.
//! Also provides codec compatibility checking and human-readable size formatting.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Definition of a supported audio output format with FFmpeg encoder arguments.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct OutputFormat {
    /// File extension including the leading dot (e.g. `".mp3"`).
    pub ext: &'static str,
    /// FFmpeg codec arguments passed after `-i <input>` (e.g. `["-codec:a", "libmp3lame", "-b:a", "192k"]`).
    pub args: &'static [&'static str],
    /// Category string used for filtering (e.g. `"lossless"`, `"compressed"`).
    pub cat: &'static str,
    /// Short human-readable description shown in the CLI format list.
    pub desc: &'static str,
}

/// Map of all 106 supported output formats with their FFmpeg arguments.
pub static OUTPUT_FORMATS: LazyLock<HashMap<&'static str, OutputFormat>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Helper macro to reduce boilerplate
    macro_rules! fmt {
        ($key:expr, $ext:expr, [$($arg:expr),*], $cat:expr, $desc:expr) => {
            m.insert($key, OutputFormat {
                ext: $ext,
                args: &[$($arg),*],
                cat: $cat,
                desc: $desc,
            });
        };
    }

    fmt!(".aac", ".aac", ["-codec:a", "aac", "-b:a", "192k"], "compressed", "AAC 192kbps");
    fmt!(".ac3", ".ac3", ["-codec:a", "ac3", "-b:a", "448k"], "compressed", "Dolby Digital 5.1");
    fmt!(".adts", ".adts", ["-codec:a", "aac", "-b:a", "192k", "-f", "adts"], "compressed", "AAC ADTS");
    fmt!(".aif", ".aif", ["-codec:a", "pcm_s16be", "-ar", "44100", "-f", "aiff"], "lossless", "Apple/SGI 16-bit");
    fmt!(".aifc", ".aifc", ["-codec:a", "pcm_s16be", "-ar", "44100", "-f", "aiff"], "lossless", "AIFF-C compressed");
    fmt!(".aiff", ".aiff", ["-codec:a", "pcm_s16be", "-ar", "44100", "-f", "aiff"], "lossless", "Apple/SGI 16-bit");
    fmt!(".alac", ".alac", ["-codec:a", "alac", "-f", "ipod"], "lossless", "Apple Lossless");
    fmt!(".amr", ".amr", ["-codec:a", "libopencore_amrnb", "-ar", "8000", "-ac", "1", "-f", "amr"], "compressed", "AMR-NB celular");
    fmt!(".amrnb", ".amrnb", ["-codec:a", "libopencore_amrnb", "-ar", "8000", "-ac", "1", "-f", "amr"], "compressed", "AMR NarrowBand");
    fmt!(".amrwb", ".amrwb", ["-codec:a", "libvo_amrwbenc", "-ar", "16000", "-ac", "1", "-f", "amr"], "compressed", "AMR WideBand");
    fmt!(".ape", ".ape", ["-codec:a", "flac", "-f", "flac"], "lossless", "Monkey's Audio (FLAC compat.)");
    fmt!(".au", ".au", ["-codec:a", "pcm_s16be", "-ar", "44100", "-f", "au"], "lossless", "Sun/NeXT");
    fmt!(".avr", ".avr", ["-codec:a", "pcm_s16be", "-ar", "44100", "-f", "au"], "lossless", "Audio Visual Research");
    fmt!(".caf", ".caf", ["-codec:a", "pcm_s16le", "-f", "caf"], "lossless", "Core Audio");
    fmt!(".cdda", ".cdda", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "CD Digital Audio");
    fmt!(".dff", ".dff", ["-codec:a", "pcm_s24le", "-ar", "96000", "-f", "wav"], "lossless", "DSD interop (PCM 24-bit)");
    fmt!(".dsf", ".dsf", ["-codec:a", "pcm_s24le", "-ar", "96000", "-f", "wav"], "lossless", "DSD bit-perfect (PCM 24-bit)");
    fmt!(".dts", ".dts", ["-codec:a", "dca", "-b:a", "1536k", "-strict", "-2", "-f", "dts"], "compressed", "DTS Surround");
    fmt!(".dtshd", ".dtshd", ["-codec:a", "dca", "-b:a", "2450k", "-strict", "-2", "-f", "dts"], "compressed", "DTS-HD");
    fmt!(".eac3", ".eac3", ["-codec:a", "eac3", "-b:a", "640k", "-f", "eac3"], "compressed", "Enhanced AC-3");
    fmt!(".f32be", ".f32be", ["-codec:a", "pcm_f32be", "-f", "f32be"], "lossless", "32-bit float BE");
    fmt!(".f32le", ".f32le", ["-codec:a", "pcm_f32le", "-f", "f32le"], "lossless", "32-bit float LE");
    fmt!(".f64be", ".f64be", ["-codec:a", "pcm_f64be", "-f", "f64be"], "lossless", "64-bit float BE");
    fmt!(".f64le", ".f64le", ["-codec:a", "pcm_f64le", "-f", "f64le"], "lossless", "64-bit float LE");
    fmt!(".flac", ".flac", ["-codec:a", "flac", "-f", "flac"], "lossless", "FLAC sem perda");
    fmt!(".g722", ".g722", ["-codec:a", "g722", "-ar", "16000", "-ac", "1", "-f", "g722"], "compressed", "ITU-T G.722");
    fmt!(".g723_1", ".g723_1", ["-codec:a", "g723_1", "-ar", "8000", "-ac", "1", "-f", "g723_1"], "compressed", "ITU-T G.723.1");
    fmt!(".g726", ".g726", ["-codec:a", "g726", "-ar", "8000", "-ac", "1", "-f", "g726"], "compressed", "ITU-T G.726");
    fmt!(".gsm", ".gsm", ["-codec:a", "libgsm", "-ar", "8000", "-ac", "1", "-f", "gsm"], "compressed", "GSM 06.10");
    fmt!(".ilbc", ".ilbc", ["-codec:a", "libilbc", "-ar", "8000", "-ac", "1", "-f", "ilbc"], "compressed", "iLBC VoIP");
    fmt!(".ircam", ".ircam", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "ircam"], "lossless", "IRCAM/BURT");
    fmt!(".m4a", ".m4a", ["-codec:a", "aac", "-b:a", "256k", "-f", "ipod"], "compressed", "AAC 256kbps");
    fmt!(".m4b", ".m4b", ["-codec:a", "aac", "-b:a", "128k", "-f", "ipod"], "compressed", "Audiobook AAC");
    fmt!(".m4p", ".m4p", ["-codec:a", "aac", "-b:a", "192k", "-f", "ipod"], "compressed", "AAC protegido");
    fmt!(".m4r", ".m4r", ["-codec:a", "aac", "-b:a", "128k", "-f", "ipod"], "compressed", "Ringtone iPhone");
    fmt!(".mmf", ".mmf", ["-codec:a", "adpcm_yamaha", "-ar", "22050", "-ac", "1", "-f", "mmf"], "compressed", "Yamaha SMAF");
    fmt!(".mp2", ".mp2", ["-codec:a", "libtwolame", "-b:a", "384k", "-f", "mp2"], "compressed", "MPEG-1 Audio II");
    fmt!(".mp3", ".mp3", ["-codec:a", "libmp3lame", "-b:a", "192k", "-f", "mp3"], "compressed", "MP3 192kbps");
    fmt!(".nist", ".nist", ["-codec:a", "pcm_s16le", "-ar", "16000", "-f", "wav"], "lossless", "NIST Sphere (WAV compat.)");
    fmt!(".oga", ".oga", ["-codec:a", "libvorbis", "-qscale:a", "6", "-f", "ogg"], "compressed", "OGG Vorbis audio");
    fmt!(".ogg", ".ogg", ["-codec:a", "libvorbis", "-qscale:a", "6", "-f", "ogg"], "compressed", "OGG Vorbis");
    fmt!(".opus", ".opus", ["-codec:a", "libopus", "-b:a", "128k", "-f", "opus"], "compressed", "Opus streaming");
    fmt!(".pcm_alaw", ".pcm_alaw", ["-codec:a", "pcm_alaw", "-ar", "8000", "-f", "alaw"], "lossless", "PCM A-law");
    fmt!(".pcm_mulaw", ".pcm_mulaw", ["-codec:a", "pcm_mulaw", "-ar", "8000", "-f", "mulaw"], "lossless", "PCM mu-law");
    fmt!(".pcm_s16be", ".pcm_s16be", ["-codec:a", "pcm_s16be", "-f", "s16be"], "lossless", "16-bit signed BE");
    fmt!(".pcm_s16le", ".pcm_s16le", ["-codec:a", "pcm_s16le", "-f", "s16le"], "lossless", "16-bit signed LE");
    fmt!(".pcm_s24be", ".pcm_s24be", ["-codec:a", "pcm_s24be", "-f", "s24be"], "lossless", "24-bit signed BE");
    fmt!(".pcm_s24le", ".pcm_s24le", ["-codec:a", "pcm_s24le", "-f", "s24le"], "lossless", "24-bit signed LE");
    fmt!(".pcm_s32be", ".pcm_s32be", ["-codec:a", "pcm_s32be", "-f", "s32be"], "lossless", "32-bit signed BE");
    fmt!(".pcm_s32le", ".pcm_s32le", ["-codec:a", "pcm_s32le", "-f", "s32le"], "lossless", "32-bit signed LE");
    fmt!(".pcm_u8", ".pcm_u8", ["-codec:a", "pcm_u8", "-ar", "8000", "-f", "u8"], "lossless", "8-bit unsigned");
    fmt!(".raw", ".raw", ["-codec:a", "pcm_s16le", "-f", "s16le"], "lossless", "Headerless PCM");
    fmt!(".s16be", ".s16be", ["-codec:a", "pcm_s16be", "-f", "s16be"], "lossless", "16-bit signed BE");
    fmt!(".s16le", ".s16le", ["-codec:a", "pcm_s16le", "-f", "s16le"], "lossless", "16-bit signed LE");
    fmt!(".s24be", ".s24be", ["-codec:a", "pcm_s24be", "-f", "s24be"], "lossless", "24-bit signed BE");
    fmt!(".s24le", ".s24le", ["-codec:a", "pcm_s24le", "-f", "s24le"], "lossless", "24-bit signed LE");
    fmt!(".s32be", ".s32be", ["-codec:a", "pcm_s32be", "-f", "s32be"], "lossless", "32-bit signed BE");
    fmt!(".s32le", ".s32le", ["-codec:a", "pcm_s32le", "-f", "s32le"], "lossless", "32-bit signed LE");
    fmt!(".sox", ".sox", ["-codec:a", "pcm_s32le", "-f", "sox"], "lossless", "SoX native");
    fmt!(".sph", ".sph", ["-codec:a", "pcm_s16le", "-ar", "16000", "-f", "wav"], "lossless", "NIST SPH (WAV compat.)");
    fmt!(".spx", ".spx", ["-codec:a", "libspeex", "-ar", "32000", "-ac", "1", "-f", "spx"], "compressed", "Speex voz");
    fmt!(".tta", ".tta", ["-codec:a", "tta", "-f", "tta"], "lossless", "True Audio");
    fmt!(".u8", ".u8", ["-codec:a", "pcm_u8", "-ar", "8000", "-f", "u8"], "lossless", "8-bit unsigned");
    fmt!(".voc", ".voc", ["-codec:a", "pcm_u8", "-ar", "8000", "-f", "voc"], "lossless", "Creative Voice");
    fmt!(".w64", ".w64", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "w64"], "lossless", "Sony Wave64");
    fmt!(".wav", ".wav", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "WAV 16-bit CD");
    fmt!(".wave", ".wave", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "WAVE 16-bit CD");
    fmt!(".wma", ".wma", ["-codec:a", "wmav2", "-b:a", "192k", "-f", "asf"], "compressed", "Windows Media 192k");
    fmt!(".wv", ".wv", ["-codec:a", "wavpack", "-f", "wv"], "lossless", "WavPack lossless");
    fmt!(".wvp", ".wvp", ["-codec:a", "wavpack", "-f", "wv"], "lossless", "WavPack hybrid");
 
    // Input-only formats (fallback to closest encoder)
    fmt!(".aa", ".aa", ["-codec:a", "aac", "-b:a", "128k", "-f", "adts"], "compressed", "Audible (AAC compat.)");
    fmt!(".acm", ".acm", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "Windows ACM (PCM)");
    fmt!(".act", ".act", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "ACT (PCM compat.)");
    fmt!(".adx", ".adx", ["-codec:a", "adpcm_adx", "-f", "adx"], "compressed", "CRI ADX (AAC compat.)");
    fmt!(".aea", ".aea", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "MD STUDIO (PCM)");
    fmt!(".apc", ".apc", ["-codec:a", "flac", "-f", "flac"], "lossless", "Crystal (FLAC compat.)");
    fmt!(".apl", ".apl", ["-codec:a", "alac", "-f", "ipod"], "lossless", "Apple Lossless (ALAC)");
    fmt!(".aud", ".aud", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "Generic audio (PCM)");
    fmt!(".g729", ".g729", ["-codec:a", "libilbc", "-ar", "8000", "-ac", "1", "-f", "ilbc"], "compressed", "ITU G.729 (iLBC compat.)");
    fmt!(".hca", ".hca", ["-codec:a", "aac", "-b:a", "128k", "-f", "adts"], "compressed", "CRI HCA (AAC compat.)");
    fmt!(".hcom", ".hcom", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "HCOM (PCM)");
    fmt!(".imc", ".imc", ["-codec:a", "aac", "-b:a", "128k", "-f", "adts"], "compressed", "IMC (AAC compat.)");
    fmt!(".iss", ".iss", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "Funcom ISS (PCM)");
    fmt!(".it", ".it", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "Impulse Tracker (PCM)");
    fmt!(".la", ".la", ["-codec:a", "flac", "-f", "flac"], "lossless", "Lossless Audio (FLAC)");
    fmt!(".mlp", ".mlp", ["-codec:a", "flac", "-f", "flac"], "lossless", "Meridian Lossless (FLAC)");
    fmt!(".mpc", ".mpc", ["-codec:a", "libmp3lame", "-qscale:a", "0", "-f", "mp3"], "compressed", "Musepack (MP3 V0)");
    fmt!(".mqa", ".mqa", ["-codec:a", "flac", "-f", "flac"], "lossless", "MQA (FLAC compat.)");
    fmt!(".oma", ".oma", ["-codec:a", "aac", "-b:a", "192k", "-f", "oma"], "compressed", "Sony OpenMG (AAC)");
    fmt!(".paf", ".paf", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "PAF (PCM)");
    fmt!(".pvf", ".pvf", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "PVF (PCM)");
    fmt!(".qcp", ".qcp", ["-codec:a", "aac", "-b:a", "128k", "-f", "adts"], "compressed", "QCP (AAC compat.)");
    fmt!(".ra", ".ra", ["-codec:a", "libmp3lame", "-b:a", "192k", "-f", "mp3"], "compressed", "RealAudio (MP3 compat.)");
    fmt!(".s3m", ".s3m", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "Screamtracker (PCM)");
    fmt!(".shn", ".shn", ["-codec:a", "flac", "-f", "flac"], "lossless", "Shorten (FLAC compat.)");
    fmt!(".sid", ".sid", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "SID (PCM compat.)");
    fmt!(".sol", ".sol", ["-codec:a", "pcm_s16le", "-f", "wav"], "lossless", "Sierra SOL (PCM)");
    fmt!(".tak", ".tak", ["-codec:a", "flac", "-f", "flac"], "lossless", "TAK (FLAC compat.)");
    fmt!(".thd", ".thd", ["-codec:a", "flac", "-f", "flac"], "lossless", "TrueHD (FLAC compat.)");
    fmt!(".vqf", ".vqf", ["-codec:a", "libmp3lame", "-b:a", "192k", "-f", "mp3"], "compressed", "Yamaha VQF (MP3)");
    fmt!(".xa", ".xa", ["-codec:a", "pcm_s16le", "-ar", "22050", "-f", "wav"], "lossless", "Maxis XA (PCM)");
    fmt!(".xm", ".xm", ["-codec:a", "pcm_s16le", "-ar", "44100", "-f", "wav"], "lossless", "FastTracker (PCM)");

    m
});

/// Set of all supported input file extensions (83 formats).
pub static INPUT_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let exts: &[&str] = &[
        ".aa", ".aac", ".ac3", ".acm", ".act", ".adts", ".adx", ".aea",
        ".aif", ".aifc", ".aiff", ".alac", ".amr", ".amrnb", ".amrwb",
        ".ape", ".apl", ".apc",
        ".au", ".aud", ".avr",
        ".caf", ".cdda",
        ".dff", ".dsf",
        ".dts", ".dtshd",
        ".eac3",
        ".f32be", ".f32le", ".f64be", ".f64le",
        ".flac",
        ".g722", ".g723_1", ".g726", ".g729",
        ".gsm", ".hca", ".hcom",
        ".ilbc", ".imc", ".ircam", ".iss",
        ".it",
        ".la",
        ".m4a", ".m4b", ".m4p", ".m4r",
        ".mlp",
        ".mmf",
        ".mp2", ".mp3", ".mpc", ".mqa",
        ".nist",
        ".oga", ".ogg", ".oma", ".opus",
        ".paf", ".pvf",
        ".pcm_alaw", ".pcm_mulaw",
        ".pcm_s16be", ".pcm_s16le",
        ".pcm_s24be", ".pcm_s24le",
        ".pcm_s32be", ".pcm_s32le",
        ".pcm_u8",
        ".qcp",
        ".ra", ".raw",
        ".s16be", ".s16le",
        ".s24be", ".s24le",
        ".s32be", ".s32le",
        ".shn",
        ".s3m", ".sid", ".sol", ".sox", ".sph", ".spx",
        ".tak", ".thd", ".tta",
        ".u8",
        ".voc", ".vqf",
        ".w64", ".wav", ".wave",
        ".wma",
        ".wv", ".wvp",
        ".xa", ".xm",
        ".snd", ".sln", ".wve",
        ".webm", ".mka", ".awb", ".vox",
        ".mid", ".midi", ".kar", ".mod", ".mtm",
        ".umx", ".mo3", ".xmq",
        ".mkv", ".avi", ".mov", ".m4v", ".flv",
        ".3gp", ".mpg", ".mpeg", ".ts", ".vob",
        ".ogv", ".f4v", ".rm", ".rmvb", ".asf", ".divx",
        ".m2ts", ".mts", ".nsv",
    ];
    exts.iter().copied().collect()
});

/// Named speed presets for playback rate adjustment.
pub static SPEED_PRESETS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("ultrafast", vec!["-preset", "ultrafast", "-threads", "0"]);
    m.insert("fast", vec!["-preset", "fast", "-threads", "0"]);
    m.insert("normal", vec!["-threads", "0"]);
    m.insert("quality", vec!["-threads", "0", "-ar", "96000"]);
    m
});

/// Standard audio sample rates in Hz.
pub static SAMPLE_RATES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Original", "0");
    m.insert("8 kHz", "8000");
    m.insert("11 kHz", "11025");
    m.insert("16 kHz", "16000");
    m.insert("22 kHz", "22050");
    m.insert("44.1 kHz", "44100");
    m.insert("48 kHz", "48000");
    m.insert("88.2 kHz", "88200");
    m.insert("96 kHz", "96000");
    m.insert("176.4 kHz", "176400");
    m.insert("192 kHz", "192000");
    m.insert("352.8 kHz", "352800");
    m.insert("384 kHz", "384000");
    m
});

/// Available channel configurations (mono, stereo, 5.1).
pub static CHANNELS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Original", "0");
    m.insert("Mono (1)", "1");
    m.insert("Stereo (2)", "2");
    m.insert("2.1", "3");
    m.insert("4.0", "4");
    m.insert("5.1", "6");
    m.insert("6.1", "7");
    m.insert("7.1", "8");
    m
});

/// Format pairs where codec copy is compatible between input and output.
pub static COPY_COMPAT: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(".mp3", vec![".mp3"]);
    m.insert(".flac", vec![".flac"]);
    m.insert(".wav", vec![".wav", ".w64"]);
    m.insert(".w64", vec![".w64", ".wav"]);
    m.insert(".ogg", vec![".ogg"]);
    m.insert(".opus", vec![".opus"]);
    m.insert(".aac", vec![".aac", ".m4a"]);
    m.insert(".m4a", vec![".m4a", ".aac"]);
    m.insert(".wma", vec![".wma"]);
    m.insert(".ape", vec![".ape"]);
    m.insert(".wv", vec![".wv"]);
    m.insert(".tta", vec![".tta"]);
    m.insert(".aiff", vec![".aiff", ".aif"]);
    m.insert(".aif", vec![".aif", ".aiff"]);
    m.insert(".dsf", vec![".dsf"]);
    m.insert(".dff", vec![".dff"]);
    m.insert(".ac3", vec![".ac3"]);
    m.insert(".dts", vec![".dts"]);
    m.insert(".mp2", vec![".mp2"]);
    m
});

/// Returns `true` if the input codec can be stream-copied to the output format.
pub fn can_copy(input_ext: &str, output_ext: &str) -> bool {
    if let Some(compatible) = COPY_COMPAT.get(input_ext) {
        return compatible.iter().any(|s| *s == output_ext);
    }
    false
}

/// Formats a byte count as a human-readable string (B, KB, MB, GB).
pub fn human_size(size_bytes: u64) -> String {
    if size_bytes < 1024 {
        format!("{} B", size_bytes)
    } else if size_bytes < 1024 * 1024 {
        format!("{:.1} KB", size_bytes as f64 / 1024.0)
    } else if size_bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
