use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoArchiveConfig {
    pub enabled: bool,
    pub completion_percent: u8,
    pub required_listens: u32,
    pub archive_dir: String,
}

impl Default for AutoArchiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            completion_percent: 90,
            required_listens: 3,
            archive_dir: "archive".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub last_folder: String,
    pub last_file: Option<String>,
    pub last_elapsed_secs: u64,
    pub resume_mode: bool,
    pub show_hidden: bool,
    #[serde(default)]
    pub auto_archive: AutoArchiveConfig,
    #[serde(default)]
    pub play_counts: HashMap<String, u32>,
    #[serde(default)]
    pub listen_times_secs: HashMap<String, f64>,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            last_folder: String::new(),
            last_file: None,
            last_elapsed_secs: 0,
            resume_mode: true, // Resume ON by default so user position is remembered
            show_hidden: false,
            auto_archive: AutoArchiveConfig::default(),
            play_counts: HashMap::new(),
            listen_times_secs: HashMap::new(),
        }
    }
}

impl PersistentState {
    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let dir = PathBuf::from(home).join(".config").join("ototune");
        let _ = fs::create_dir_all(&dir);
        Some(dir.join("state.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(state) = serde_json::from_str(&content) {
                        return state;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, json);
            }
        }
    }
}

