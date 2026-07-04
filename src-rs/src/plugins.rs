//! Plugin manager for extending converter functionality
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata and configuration for a converter plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub entry_point: Option<String>,
}

impl PluginInfo {
    /// Validates the plugin information fields.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Nome do plugin vazio".into());
        }
        if self.name.len() > 100 {
            return Err("Nome do plugin muito longo (max 100)".into());
        }
        if self.version.is_empty() {
            return Err("Versão do plugin vazia".into());
        }
        if !self.version.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c.is_ascii_alphabetic()) {
            return Err("Versão inválida (apenas alfanuméricos, ., -)".into());
        }
        if self.description.len() > 1000 {
            return Err("Descrição muito longa (max 1000)".into());
        }
        if let Some(author) = &self.author {
            if author.len() > 100 {
                return Err("Autor muito longo (max 100)".into());
            }
        }
        if let Some(entry) = &self.entry_point {
            if entry.is_empty() || entry.len() > 200 {
                return Err("Entry point inválido".into());
            }
            if entry.contains("..") || entry.contains('/') || entry.contains('\\') {
                return Err("Entry point contém caminho inválido".into());
            }
        }
        Ok(())
    }
}

/// Manages loading, validation, and lifecycle of converter plugins.
pub struct PluginManager {
    plugins_dir: PathBuf,
    plugins: Vec<PluginInfo>,
}

impl PluginManager {
    /// Creates a new `PluginManager`, ensuring the plugins directory exists.
    pub fn new() -> Self {
        let plugins_dir = crate::portable::Portable::plugins_dir();
        let _ = std::fs::create_dir_all(&plugins_dir);

        Self {
            plugins_dir,
            plugins: Vec::new(),
        }
    }

    /// Scans the plugins directory and returns paths to found plugin files.
    pub fn discover(&self) -> Vec<String> {
        let Ok(read_dir) = std::fs::read_dir(&self.plugins_dir) else {
            return Vec::new();
        };
        read_dir
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.extension()?.to_string_lossy() == "json" {
                    path.file_stem().map(|s| s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Discovers and loads all available plugins into the manager.
    pub fn load_all(&mut self) {
        self.plugins.clear();
        let found = self.discover();
        for name in found {
            if name.len() > 100 {
                continue;
            }
            if !crate::utils::is_safe_path(&name) {
                continue;
            }
            let plugin_path = self.plugins_dir.join(format!("{}.json", name));
            if let Ok(info) = crate::utils::load_json::<PluginInfo>(&plugin_path, 10240) {
                if info.validate().is_ok() {
                    self.plugins.push(info);
                }
            }
        }
    }

    /// Returns the list of currently loaded plugin metadata.
    pub fn get_plugins(&self) -> &[PluginInfo] {
        &self.plugins
    }
}
