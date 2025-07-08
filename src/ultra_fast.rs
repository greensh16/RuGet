use crate::error::{Result, RuGetError};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Ultra-fast download that bypasses as much overhead as possible
/// This is specifically optimized for small files and simple HTTP downloads
pub fn ultra_fast_download(url: &str, output_path: &str) -> Result<()> {
    // Check if output file already exists to avoid unnecessary work
    if Path::new(output_path).exists() {
        return Ok(());
    }

    // Create parent directories if needed (but only if necessary)
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Use the most minimal HTTP client possible
    let data = ultra_minimal_get(url)?;
    
    // Write with buffered writer for better performance
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&data)?;
    writer.flush()?;
    
    Ok(())
}

/// Absolute minimal HTTP GET - optimized for speed over features
fn ultra_minimal_get(url: &str) -> Result<Vec<u8>> {
    // For maximum performance, use ureq which is much lighter than reqwest
    // Note: This requires adding ureq to Cargo.toml
    
    // For now, use a highly optimized reqwest configuration
    use std::time::Duration;
    
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))  // Shorter timeout
        .pool_idle_timeout(Duration::from_secs(0))  // Minimal connection pooling
        .pool_max_idle_per_host(0)         // No connection pooling
        .tcp_nodelay(true)                 // Disable Nagle's algorithm
        .build()?;
    
    let response = client
        .get(url)
        .header("Connection", "close")     // Don't keep connection alive
        .send()?;
    
    if !response.status().is_success() {
        return Err(RuGetError::network(format!("HTTP {}", response.status())));
    }
    
    // Read directly into Vec for minimal copying
    Ok(response.bytes()?.to_vec())
}

/// Check if we can use ultra-fast mode
pub fn can_use_ultra_fast(url: &str, output_path: &str) -> bool {
    // Only for HTTPS URLs (HTTP is less common and less secure)
    url.starts_with("https://") 
        && !output_path.is_empty()
        && !url.contains('?')  // No query parameters for simplicity
        && url.len() < 200     // Reasonable URL length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_use_ultra_fast() {
        assert!(can_use_ultra_fast("https://example.com/file", "output.txt"));
        assert!(!can_use_ultra_fast("http://example.com/file", "output.txt"));
        assert!(!can_use_ultra_fast("https://example.com/file?param=1", "output.txt"));
        assert!(!can_use_ultra_fast("https://example.com/file", ""));
    }
}
