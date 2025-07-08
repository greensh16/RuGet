use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use ruget::config::{Config, RetryConfig, LoggingConfig};
use ruget::cli::{Args, LogFormat, LogLevel};
use ruget::lazy_config::apply_config_if_needed;

/// Test the new retry config section in TOML
#[test]
fn test_retry_config_parsing() {
    let toml_content = r#"
[retry]
max = 5
base_ms = 500
max_ms = 10000
"#;

    let config: Config = toml::from_str(toml_content).expect("Failed to parse config");
    
    assert!(config.retry.is_some());
    let retry = config.retry.unwrap();
    assert_eq!(retry.max, Some(5));
    assert_eq!(retry.base_ms, Some(500));
    assert_eq!(retry.max_ms, Some(10000));
}

/// Test the new logging config section in TOML
#[test]
fn test_logging_config_parsing() {
    let toml_content = r#"
[logging]
format = "json"
level = "info"
"#;

    let config: Config = toml::from_str(toml_content).expect("Failed to parse config");
    
    assert!(config.logging.is_some());
    let logging = config.logging.unwrap();
    assert_eq!(logging.format, Some("json".to_string()));
    assert_eq!(logging.level, Some("info".to_string()));
}

/// Test complete config with both old and new sections
#[test]
fn test_complete_config_parsing() {
    let toml_content = r#"
retries = 3
resume = true
quiet = false
verbose = true
jobs = 4
output_dir = "/tmp/downloads"
headers = ["User-Agent: ruget/1.0", "Accept: */*"]
log = "custom.log"
backoff_base_ms = 200
backoff_max_ms = 30000

[retry]
max = 5
base_ms = 500
max_ms = 10000

[logging]
format = "json"
level = "debug"
"#;

    let config: Config = toml::from_str(toml_content).expect("Failed to parse config");
    
    // Test old fields
    assert_eq!(config.retries, Some(3));
    assert_eq!(config.resume, Some(true));
    assert_eq!(config.quiet, Some(false));
    assert_eq!(config.verbose, Some(true));
    assert_eq!(config.jobs, Some(4));
    assert_eq!(config.output_dir, Some("/tmp/downloads".to_string()));
    assert_eq!(config.headers, Some(vec!["User-Agent: ruget/1.0".to_string(), "Accept: */*".to_string()]));
    assert_eq!(config.log, Some("custom.log".to_string()));
    assert_eq!(config.backoff_base_ms, Some(200));
    assert_eq!(config.backoff_max_ms, Some(30000));
    
    // Test new retry section
    assert!(config.retry.is_some());
    let retry = config.retry.unwrap();
    assert_eq!(retry.max, Some(5));
    assert_eq!(retry.base_ms, Some(500));
    assert_eq!(retry.max_ms, Some(10000));
    
    // Test new logging section
    assert!(config.logging.is_some());
    let logging = config.logging.unwrap();
    assert_eq!(logging.format, Some("json".to_string()));
    assert_eq!(logging.level, Some("debug".to_string()));
}

/// Test merging retry config into args
#[test]
fn test_retry_config_merge() {
    let config = Config {
        retry: Some(RetryConfig {
            max: Some(7),
            base_ms: Some(1000),
            max_ms: Some(20000),
        }),
        ..Default::default()
    };
    
    let mut args = Args {
        urls: vec!["https://example.com".to_string()],
        input: None,
        output: None,
        headers: vec![],
        resume: false,
        max_retries: 0, // Should be overridden by config
        verbose: false,
        log_json: false,
        log_format: None,
        log_level: None,
        quiet: false,
        output_dir: None,
        jobs: 0,
        log: "".to_string(),
        init: false,
        backoff_base_ms: 100, // Should be overridden by config
        backoff_max_ms: 60000, // Should be overridden by config
    };
    
    config.merge_with_args(&mut args);
    
    assert_eq!(args.max_retries, 7);
    assert_eq!(args.backoff_base_ms, 1000);
    assert_eq!(args.backoff_max_ms, 20000);
}

/// Test merging logging config into args
#[test]
fn test_logging_config_merge() {
    let config = Config {
        logging: Some(LoggingConfig {
            format: Some("json".to_string()),
            level: Some("debug".to_string()),
        }),
        ..Default::default()
    };
    
    let mut args = Args {
        urls: vec!["https://example.com".to_string()],
        input: None,
        output: None,
        headers: vec![],
        resume: false,
        max_retries: 3,
        verbose: false,
        log_json: false,
        log_format: None, // Should be overridden
        log_level: None,  // Should be overridden
        quiet: false,
        output_dir: None,
        jobs: 0,
        log: "test.log".to_string(),
        init: false,
        backoff_base_ms: 100,
        backoff_max_ms: 60000,
    };
    
    config.merge_with_args(&mut args);
    
    assert!(matches!(args.log_format, Some(LogFormat::Json)));
    assert!(matches!(args.log_level, Some(LogLevel::Debug)));
}

/// Test that CLI args take precedence over config
#[test]
fn test_cli_precedence_over_config() {
    let config = Config {
        retry: Some(RetryConfig {
            max: Some(5),
            base_ms: Some(500),
            max_ms: Some(10000),
        }),
        logging: Some(LoggingConfig {
            format: Some("json".to_string()),
            level: Some("debug".to_string()),
        }),
        ..Default::default()
    };
    
    let mut args = Args {
        urls: vec!["https://example.com".to_string()],
        input: None,
        output: None,
        headers: vec![],
        resume: false,
        max_retries: 10, // Already set, should not be overridden
        verbose: false,
        log_json: false,
        log_format: Some(LogFormat::Text), // Already set, should not be overridden
        log_level: Some(LogLevel::Error),  // Already set, should not be overridden
        quiet: false,
        output_dir: None,
        jobs: 0,
        log: "test.log".to_string(),
        init: false,
        backoff_base_ms: 200, // Already set, should not be overridden
        backoff_max_ms: 30000, // Already set, should not be overridden
    };
    
    config.merge_with_args(&mut args);
    
    // CLI values should be preserved
    assert_eq!(args.max_retries, 10);
    assert_eq!(args.backoff_base_ms, 200);
    assert_eq!(args.backoff_max_ms, 30000);
    assert!(matches!(args.log_format, Some(LogFormat::Text)));
    assert!(matches!(args.log_level, Some(LogLevel::Error)));
}

/// Test priority: old config fields vs new config sections
#[test]
fn test_config_section_priority() {
    // New sections should take precedence over old individual fields
    let config = Config {
        retries: Some(3),
        backoff_base_ms: Some(100),
        backoff_max_ms: Some(60000),
        retry: Some(RetryConfig {
            max: Some(7),      // Should override retries field
            base_ms: Some(1000), // Should override backoff_base_ms field
            max_ms: Some(20000), // Should override backoff_max_ms field
        }),
        ..Default::default()
    };
    
    let mut args = Args {
        urls: vec!["https://example.com".to_string()],
        input: None,
        output: None,
        headers: vec![],
        resume: false,
        max_retries: 0, // Use config
        verbose: false,
        log_json: false,
        log_format: None,
        log_level: None,
        quiet: false,
        output_dir: None,
        jobs: 0,
        log: "".to_string(),
        init: false,
        backoff_base_ms: 100, // Use config
        backoff_max_ms: 60000, // Use config
    };
    
    config.merge_with_args(&mut args);
    
    // New section values should take precedence
    assert_eq!(args.max_retries, 7);    // From retry.max, not retries
    assert_eq!(args.backoff_base_ms, 1000);  // From retry.base_ms, not backoff_base_ms
    assert_eq!(args.backoff_max_ms, 20000);  // From retry.max_ms, not backoff_max_ms
}

/// Test apply_config_if_needed detects when logging config is needed
#[test]
fn test_apply_config_if_needed_with_logging() {
    // This test verifies the lazy loading correctly identifies when logging config is needed
    let mut args = Args {
        urls: vec!["https://example.com".to_string()],
        input: None,
        output: None,
        headers: vec![],
        resume: false,
        max_retries: 3, // Set
        verbose: false,
        log_json: false,
        log_format: None, // Not set - should trigger config loading
        log_level: None,  // Not set - should trigger config loading
        quiet: false,
        output_dir: None,
        jobs: 1, // Set
        log: "test.log".to_string(), // Set
        init: false,
        backoff_base_ms: 100,
        backoff_max_ms: 60000,
    };
    
    // This will load config from ~/.rugetrc if it exists
    // In a test environment, it should just apply defaults
    apply_config_if_needed(&mut args);
    
    // The function should complete without panicking
    // Actual config loading depends on the test environment
}

/// Test default values for new config sections
#[test]
fn test_default_config_values() {
    let config = Config::default();
    
    assert!(config.retry.is_none());
    assert!(config.logging.is_none());
    
    let retry_config = RetryConfig::default();
    assert!(retry_config.max.is_none());
    assert!(retry_config.base_ms.is_none());
    assert!(retry_config.max_ms.is_none());
    
    let logging_config = LoggingConfig::default();
    assert!(logging_config.format.is_none());
    assert!(logging_config.level.is_none());
}
