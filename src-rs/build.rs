//! Build Script
//!
//! Embeds the application icon into the Windows executable using winres.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>

fn main() {
    // Ensure the dist directory exists so cargo compiles successfully
    // even if npm run build hasn't been run yet (e.g. on fresh checkout).
    let dist_dir = std::path::Path::new("..").join("dist");
    if !dist_dir.exists() {
        let _ = std::fs::create_dir_all(&dist_dir);
        let _ = std::fs::write(dist_dir.join("index.html"), "<html><body>Trix Frontend placeholder</body></html>");
    }

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("..\\assets\\icons\\trix_logo_sunset.ico");
        res.set("ProductName", "Trix Audio Converter");
        res.set("FileDescription", "Trix Audio Converter — Conversor de áudio universal");
        res.set("LegalCopyright", "Copyright © 2026 João Vitor de Melo");
        res.set("CompanyName", "Trix Software");
        res.set("FileVersion", "1.0.0");
        res.set("ProductVersion", "1.0.0");
        res.compile().unwrap();
    }
}
