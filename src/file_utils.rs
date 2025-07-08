use std::fs;
use crate::error::{Result, RuGetError, WithContext};

#[cfg(feature = "context")]
use crate::error::AnyhowContextExt;

/// Load URLs from an input file
/// Filters out empty lines and comments (lines starting with #)
pub fn load_urls_from_file(path: &str) -> Result<Vec<String>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("reading input file '{}'", path))?;

    let urls: Vec<String> = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect();

    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_urls_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "https://example.com/file1.txt").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "https://example.com/file2.txt").unwrap();
        writeln!(temp_file, "  https://example.com/file3.txt  ").unwrap();

        let urls = load_urls_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://example.com/file1.txt");
        assert_eq!(urls[1], "https://example.com/file2.txt");
        assert_eq!(urls[2], "https://example.com/file3.txt");
    }

    #[test]
    fn test_load_urls_from_nonexistent_file() {
        let result = load_urls_from_file("/nonexistent/file.txt");
        assert!(result.is_err());
    }
}
