use crate::config::Config;
use crate::cli::Args;
use std::sync::OnceLock;

/// Global lazy-loaded configuration
static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

/// Get configuration, loading it only when first accessed
pub fn get_config() -> &'static Config {
    GLOBAL_CONFIG.get_or_init(|| {
        Config::from_file()
    })
}

/// Apply configuration to args only if needed (lazy approach)
pub fn apply_config_if_needed(args: &mut Args) {
    // Only load config if we have default values that might need overriding
    let needs_config = args.max_retries == 0
        || !args.resume 
        || !args.quiet 
        || !args.verbose 
        || args.jobs == 0 
        || args.output_dir.is_none() 
        || args.headers.is_empty() 
        || args.log.is_empty()
        || args.log_format.is_none()
        || args.log_level.is_none();
        
    if needs_config {
        let config = get_config();
        config.merge_with_args(args);
    }
}

/// Fast path: skip config loading entirely for simple cases
pub fn skip_config_for_simple_download(args: &Args) -> bool {
    // If all important settings are explicitly set, skip config
    args.max_retries > 0
        && args.output.is_some()
        && (args.quiet || !args.verbose)
        && args.headers.is_empty()
        && args.urls.len() == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_config_for_simple_download() {
        let args = Args {
            urls: vec!["https://example.com".to_string()],
            input: None,
            output: Some("test.txt".to_string()),
            headers: vec![],
            resume: false,
            max_retries: 3,
            verbose: false,
            log_json: false,
            log_format: Some(crate::cli::LogFormat::Text),
            log_level: Some(crate::cli::LogLevel::Info),
            quiet: true,
            output_dir: None,
            jobs: 0,
            log: "test.log".to_string(),
            init: false,
            backoff_base_ms: 100,
            backoff_max_ms: 60000,
            load_cookies: None,
            save_cookies: None,
            keep_session_cookies: false,
        };
        
        assert!(skip_config_for_simple_download(&args));
    }
}
