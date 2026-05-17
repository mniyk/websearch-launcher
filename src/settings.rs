use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const KEYRING_SERVICE: &str = "websearch-launcher";
const KEYRING_USER: &str = "tavily-api-key";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub ollama_host: String,
    pub model_name: String,
    /// Credential Manager から読み込まれる（JSON には保存しない）
    #[serde(skip)]
    pub tavily_api_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost:11434".to_string(),
            model_name: "qwen2.5:7b".to_string(),
            tavily_api_key: String::new(),
        }
    }
}

/// Credential Manager のキーを優先し、空なら環境変数 TAVILY_API_KEY にフォールバック
pub fn resolve_tavily_key(settings: &Settings) -> String {
    if !settings.tavily_api_key.is_empty() {
        return settings.tavily_api_key.clone();
    }
    std::env::var("TAVILY_API_KEY").unwrap_or_default()
}

fn load_api_key() -> String {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .ok()
        .and_then(|e| e.get_password().ok())
        .unwrap_or_default()
}

fn save_api_key(key: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    if key.is_empty() {
        entry.delete_credential().ok(); // 存在しなくてもエラーにしない
    } else {
        entry.set_password(key)?;
    }
    Ok(())
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("websearch-launcher");
    path.push("settings.json");
    path
}

pub fn load() -> Settings {
    let path = config_path();
    let mut settings = match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Settings::default(),
    };
    settings.tavily_api_key = load_api_key();
    settings
}

pub fn save(settings: &Settings) -> Result<()> {
    // API キーは Credential Manager へ
    save_api_key(&settings.tavily_api_key)?;

    // 残りの設定を JSON へ（tavily_api_key は #[serde(skip)] で除外される）
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, content)?;
    Ok(())
}
