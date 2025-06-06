use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub retries: Option<usize>,
    pub resume: Option<bool>,
    pub quiet: Option<bool>,
    pub verbose: Option<bool>,
    pub jobs: Option<usize>,
    pub output_dir: Option<String>,
    pub headers: Option<Vec<String>>,
    pub log: Option<String>,
}

impl Config {
    pub fn from_file() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let config_path = PathBuf::from(format!("{}/.rugetrc", home));
        if let Ok(content) = fs::read_to_string(config_path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            Config::default()
        }
    }
}