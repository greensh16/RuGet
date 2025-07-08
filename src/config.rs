use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use crate::error::{Result, RuGetError, WithContext};

#[cfg(feature = "context")]
use crate::error::AnyhowContextExt;

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
    pub backoff_base_ms: Option<u64>,
    pub backoff_max_ms: Option<u64>,
    pub retry: Option<RetryConfig>,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RetryConfig {
    pub max: Option<usize>,
    pub base_ms: Option<u64>,
    pub max_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LoggingConfig {
    pub format: Option<String>,
    pub level: Option<String>,
}

impl Config {
    /// Load configuration from ~/.rugetrc file
    /// Returns default config if file doesn't exist or has errors
    pub fn from_file() -> Self {
        match Self::load_from_file() {
            Ok(config) => config,
            Err(_) => {
                // Log warning but continue with default config
                Config::default()
            }
        }
    }

    /// Load configuration from ~/.rugetrc file with error propagation
    pub fn load_from_file() -> Result<Self> {
        let home = std::env::var("HOME")
            .with_context(|| "reading HOME environment variable for config".to_string())?;
        
        let config_path = PathBuf::from(format!("{}/.rugetrc", home));
        
        if !config_path.exists() {
            return Ok(Config::default());
        }
        
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("reading config file {}", config_path.display()))?;
        
        toml::from_str(&content)
            .with_context(|| format!("parsing config file {}", config_path.display()))
    }

    /// Get the effective configuration by merging with command line args
    pub fn merge_with_args(&self, args: &mut crate::cli::Args) {
        // Handle new retry config section first (takes precedence over old fields)
        if let Some(retry_config) = &self.retry {
            if args.max_retries == 0 {
                args.max_retries = retry_config.max.unwrap_or(3) as u32;
            }
            if args.backoff_base_ms == 100 && retry_config.base_ms.is_some() {
                args.backoff_base_ms = retry_config.base_ms.unwrap();
            }
            if args.backoff_max_ms == 60000 && retry_config.max_ms.is_some() {
                args.backoff_max_ms = retry_config.max_ms.unwrap();
            }
        } else {
            // Apply old fields only if new section doesn't exist
            if args.max_retries == 0 {
                args.max_retries = self.retries.unwrap_or(3) as u32;
            }
            if args.backoff_base_ms == 100 {
                args.backoff_base_ms = self.backoff_base_ms.unwrap_or(100);
            }
            if args.backoff_max_ms == 60000 {
                args.backoff_max_ms = self.backoff_max_ms.unwrap_or(60000);
            }
        }
        
        // Handle other config fields (not related to retry)
        if !args.resume {
            args.resume = self.resume.unwrap_or(false);
        }
        if !args.quiet {
            args.quiet = self.quiet.unwrap_or(false);
        }
        if !args.verbose {
            args.verbose = self.verbose.unwrap_or(false);
        }
        if args.jobs == 0 {
            args.jobs = self.jobs.unwrap_or(0);
        }
        if args.output_dir.is_none() {
            args.output_dir = self.output_dir.clone();
        }
        if args.headers.is_empty() {
            args.headers = self.headers.clone().unwrap_or_default();
        }
        if args.log.is_empty() {
            args.log = self.log.clone().unwrap_or_else(|| "ruget_failures.log".to_string());
        }
        
        // Handle new logging config section
        if let Some(logging_config) = &self.logging {
            if args.log_format.is_none() {
                args.log_format = logging_config.format.as_ref().and_then(|f| {
                    match f.to_lowercase().as_str() {
                        "json" => Some(crate::cli::LogFormat::Json),
                        "text" => Some(crate::cli::LogFormat::Text),
                        _ => None,
                    }
                });
            }
            if args.log_level.is_none() {
                args.log_level = logging_config.level.as_ref().and_then(|l| {
                    match l.to_lowercase().as_str() {
                        "debug" => Some(crate::cli::LogLevel::Debug),
                        "info" => Some(crate::cli::LogLevel::Info),
                        "warn" => Some(crate::cli::LogLevel::Warn),
                        "error" => Some(crate::cli::LogLevel::Error),
                        _ => None,
                    }
                });
            }
        }
    }
}
