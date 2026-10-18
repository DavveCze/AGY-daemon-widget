use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub custom_binary: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            custom_binary: None,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_poll_interval() -> u64 {
    1000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexProviderConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default)]
    pub custom_binary: Option<String>,
}

fn default_false() -> bool {
    false
}

impl Default for CodexProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            custom_binary: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersMap {
    #[serde(default)]
    pub antigravity: ProviderConfig,
    #[serde(default)]
    pub claude: ProviderConfig,
    #[serde(default)]
    pub codex: CodexProviderConfig,
}

impl Default for ProvidersMap {
    fn default() -> Self {
        Self {
            antigravity: ProviderConfig {
                enabled: true,
                custom_binary: None,
            },
            claude: ProviderConfig {
                enabled: true,
                custom_binary: None,
            },
            codex: CodexProviderConfig {
                enabled: false,
                custom_binary: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentiComConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default = "default_true")]
    pub notification_sound: bool,
    #[serde(default)]
    pub providers: ProvidersMap,
}

impl Default for AgentiComConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            notifications_enabled: true,
            notification_sound: true,
            providers: ProvidersMap::default(),
        }
    }
}

impl AgentiComConfig {
    pub fn get_default_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/david".to_string());
        PathBuf::from(home).join(".config").join("agenticom").join("config.json")
    }

    pub fn load_or_default(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(cfg) = serde_json::from_str::<AgentiComConfig>(&content) {
                return cfg;
            }
        }
        let def = Self::default();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(serialized) = serde_json::to_string_pretty(&def) {
            let _ = fs::write(path, serialized);
        }
        def
    }
}
