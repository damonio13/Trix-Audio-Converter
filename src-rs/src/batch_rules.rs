//! Batch processing rules and conditional conversions.
//!
//! Defines rule-based processing pipelines that apply format conversions,
//! effects, and metadata operations based on file conditions.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A batch processing rule with a condition and corresponding action.
pub struct BatchRule {
    pub name: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub enabled: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines the condition that triggers a batch rule.
pub struct RuleCondition {
    pub condition_type: String,
    pub value: String,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines the action to perform when a rule condition is met.
pub struct RuleAction {
    pub action_type: String,
    pub params: std::collections::HashMap<String, String>,
}

fn compare_numeric(val: Option<&String>, target: &str, op: &str) -> bool {
    let value: u32 = val.and_then(|v| v.parse().ok()).unwrap_or(0);
    let target: u32 = target.parse().unwrap_or(0);
    match op {
        "less_than" => value < target,
        "greater_than" => value > target,
        "equals" => value == target,
        "not_equals" => value != target,
        _ => false,
    }
}

/// Evaluates batch processing rules against audio files.
pub struct BatchRulesEngine {
    store: crate::utils::JsonStore<BatchRule>,
}

impl BatchRulesEngine {
    /// Creates a new batch rules engine.
    pub fn new() -> Self {
        let dir = crate::portable::Portable::data_dir().join("rules");
        let _ = std::fs::create_dir_all(&dir);

        Self {
            store: crate::utils::JsonStore::new(dir, 10240),
        }
    }

    /// Saves a batch rule to the store.
    pub fn add_rule(&self, rule: &BatchRule) -> Result<(), String> {
        self.store.save(&rule.name, rule)
    }

    /// Loads all rules sorted by priority (highest first).
    pub fn load_rules(&self) -> Vec<BatchRule> {
        let mut rules = self.store.load_all();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        rules
    }

    /// Evaluates all enabled rules against a file and returns matching ones.
    pub fn evaluate(&self, file_path: &str, metadata: &std::collections::HashMap<String, String>) -> Vec<BatchRule> {
        let rules = self.load_rules();
        let mut matching = Vec::new();

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            let matches = match rule.condition.condition_type.as_str() {
                "extension" => {
                    let ext = std::path::Path::new(file_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    match rule.condition.operator.as_str() {
                        "equals" => ext == rule.condition.value,
                        "contains" => ext.contains(&rule.condition.value),
                        _ => false,
                    }
                }
                "bitrate" => compare_numeric(metadata.get("bitrate"), &rule.condition.value, &rule.condition.operator),
                "sample_rate" => compare_numeric(metadata.get("sample_rate"), &rule.condition.value, &rule.condition.operator),
                "channels" => compare_numeric(metadata.get("channels"), &rule.condition.value, &rule.condition.operator),
                "filename_contains" => {
                    let filename = std::path::Path::new(file_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    filename.contains(&rule.condition.value)
                }
                _ => false,
            };

            if matches {
                matching.push(rule);
            }
        }

        matching
    }

    /// Deletes a batch rule by name.
    pub fn delete_rule(&self, name: &str) -> Result<(), String> {
        self.store.delete(name)
    }

    /// Returns a set of default batch rules for common conversions.
    pub fn get_default_rules() -> Vec<BatchRule> {
        let mut params = std::collections::HashMap::new();
        params.insert("format".into(), "mp3".into());
        params.insert("bitrate".into(), "320".into());

        vec![
            BatchRule {
                name: "FLAC to MP3 320".into(),
                condition: RuleCondition {
                    condition_type: "extension".into(),
                    value: "flac".into(),
                    operator: "equals".into(),
                },
                action: RuleAction {
                    action_type: "convert".into(),
                    params,
                },
                enabled: true,
                priority: 10,
            },
        ]
    }
}
