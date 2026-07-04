//! Build Script
//!
//! Embeds the application icon into the Windows executable using winres.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>

fn main() {
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
