use std::env;
use std::fs;
use crate::error::{Result, RuGetError};
use crate::output::Logger;

const DEFAULT_CONFIG_TEMPLATE: &str = r#"# ~/.rugetrc

# Default retry count
retries = 3

# Resume partial downloads if possible
resume = true

# Suppress output
quiet = false

# Verbose output (headers, etc)
verbose = false

# Number of parallel jobs (0 = auto)
jobs = 0

# Output directory for downloads
output_dir = "/path/to/save"

# Custom headers to send
headers = [
  "User-Agent: RuGet/1.0",
  "Accept: */*"
]

# Log file path for failed downloads
log = "/tmp/ruget_failures.log"

# Retry policy configuration
[retry]
max = 5           # Maximum number of retries
base_ms = 500     # Base delay in milliseconds
max_ms = 10000    # Maximum delay in milliseconds

# Logging configuration
[logging]
format = "text"   # Output format: "text" or "json"
level = "info"    # Log level: "debug", "info", "warn", "error"
"#;

/// Initialize a default configuration file at ~/.rugetrc
pub fn init_config(logger: &Logger) -> Result<()> {
    let home = env::var("HOME")
        .map_err(|_| RuGetError::config("HOME environment variable not set".into()))?;
    
    let config_path = format!("{}/.rugetrc", home);
    
    // Check if config already exists
    if fs::metadata(&config_path).is_ok() {
        logger.warn(&format!("Configuration file already exists at {}", config_path));
        return Ok(());
    }
    
    fs::write(&config_path, DEFAULT_CONFIG_TEMPLATE)
        .map_err(|e| RuGetError::config(format!("Failed to write config file {}: {}", config_path, e)))?;
    
    logger.info(&format!("Created configuration template at {}", config_path));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::env;

    #[test]
    fn test_init_config() {
        let temp_dir = tempdir().unwrap();
        let temp_home = temp_dir.path().to_str().unwrap();
        
        // Temporarily set HOME to our test directory
        let original_home = env::var("HOME").ok();
        unsafe { env::set_var("HOME", temp_home); }
        
        let logger = Logger::new(true, false); // quiet mode for test
        let result = init_config(&logger);
        
        // Restore original HOME
        if let Some(home) = original_home {
            unsafe { env::set_var("HOME", home); }
        } else {
            unsafe { env::remove_var("HOME"); }
        }
        
        assert!(result.is_ok());
        
        let config_path = format!("{}/.rugetrc", temp_home);
        assert!(fs::metadata(&config_path).is_ok());
        
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("retries = 3"));
    }
}
