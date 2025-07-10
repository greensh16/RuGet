use crate::cli::Args;
use crate::error::{Result, RuGetError};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Ultra-fast download for single files using minimal dependencies
/// This bypasses most of the heavy infrastructure for simple downloads
pub fn fast_single_download(url: &str, output_path: Option<&str>, quiet: bool) -> Result<()> {
    // Check if we can use the fast path
    if !is_simple_http_url(url) {
        return Err(RuGetError::network("Fast path only supports simple HTTP/HTTPS URLs".into()));
    }

    match output_path {
        Some(path) => {
            // Try ultimate mode first (native HTTP + SIMD + caching) for file output
            if crate::ultimate_fast::should_use_ultimate_mode(url, path) {
                crate::ultimate_fast::ultimate_download(url, path)?;
                if !quiet {
                    println!("Downloaded {} to {}", url, path);
                }
                return Ok(());
            }
            
            // Fall back to ultra-fast mode for file output
            if crate::ultra_fast::can_use_ultra_fast(url, path) {
                crate::ultra_fast::ultra_fast_download(url, path)?;
                if !quiet {
                    println!("Downloaded {} to {}", url, path);
                }
                return Ok(());
            }
            
            // Create output directory if needed
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Use a minimal HTTP client and write to file
            let response = simple_http_get(url)?;
            let mut file = File::create(path)?;
            file.write_all(&response)?;
            
            if !quiet {
                println!("Downloaded {} to {}", url, path);
            }
        }
        None => {
            // Output to stdout - use minimal HTTP client
            let response = simple_http_get(url)?;
            io::stdout().write_all(&response)?;
            io::stdout().flush()?;
        }
    }
    
    Ok(())
}

/// Check if URL is suitable for fast path
fn is_simple_http_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Minimal HTTP GET implementation using std library when possible
fn simple_http_get(url: &str) -> Result<Vec<u8>> {
    use crate::minimal_http::MinimalHttpClient;
    use std::time::Duration;
    
    let client = MinimalHttpClient::with_timeout(Duration::from_secs(30));
    client.get(url)
}

/// Determine if we should use fast path based on arguments
pub fn should_use_fast_path(args: &Args) -> bool {
    // Use fast path for single URL with minimal options
    args.urls.len() == 1 
        && args.input.is_none()
        && !args.resume
        && !args.verbose
        && args.headers.is_empty()
        && args.jobs <= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_simple_http_url() {
        assert!(is_simple_http_url("http://example.com"));
        assert!(is_simple_http_url("https://example.com/file"));
        assert!(!is_simple_http_url("ftp://example.com"));
        assert!(!is_simple_http_url("file:///path"));
    }

    #[test]
    fn test_should_use_fast_path() {
        let mut args = Args {
            urls: vec!["https://example.com".to_string()],
            input: None,
            output: Some("test.txt".to_string()),
            headers: vec![],
            resume: false,
            max_retries: 0,
            verbose: false,
            log_json: false,
            log_format: None,
            log_level: None,
            quiet: true,
            output_dir: None,
            jobs: 0,
            log: "test.log".to_string(),
            init: false,
            backoff_base_ms: 100,
            backoff_max_ms: 60000,
        };
        
        assert!(should_use_fast_path(&args));
        
        // Multiple URLs should not use fast path
        args.urls.push("https://example2.com".to_string());
        assert!(!should_use_fast_path(&args));
    }
}
