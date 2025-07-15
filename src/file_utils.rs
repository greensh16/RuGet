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

/// Tool-specific download command builder
/// Returns the exact CLI command for each download tool
pub fn download_cmd(tool: &str, url: &str, file: &str) -> Result<String> {
    match tool {
        "ruget" => Ok(format!("\"$RUGET_BIN\" \"{}\" --output \"{}\" --quiet", url, file)),
        "curl" => Ok(format!("\"$CURL_BIN\" -s -o \"{}\" \"{}\"", file, url)),
        "wget" => Ok(format!("\"$WGET_BIN\" -q -O \"{}\" \"{}\"", file, url)),
        _ => Err(RuGetError::parse(format!("Unknown tool '{}'. Supported tools: ruget, curl, wget", tool)))
    }
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

    #[test]
    fn test_download_cmd_ruget() {
        let cmd = download_cmd("ruget", "https://example.com/file.txt", "output.txt").unwrap();
        assert_eq!(cmd, "\"$RUGET_BIN\" \"https://example.com/file.txt\" --output \"output.txt\" --quiet");
    }

    #[test]
    fn test_download_cmd_curl() {
        let cmd = download_cmd("curl", "https://example.com/file.txt", "output.txt").unwrap();
        assert_eq!(cmd, "\"$CURL_BIN\" -s -o \"output.txt\" \"https://example.com/file.txt\"");
    }

    #[test]
    fn test_download_cmd_wget() {
        let cmd = download_cmd("wget", "https://example.com/file.txt", "output.txt").unwrap();
        assert_eq!(cmd, "\"$WGET_BIN\" -q -O \"output.txt\" \"https://example.com/file.txt\"");
    }

    #[test]
    fn test_download_cmd_unknown_tool() {
        let result = download_cmd("unknown", "https://example.com/file.txt", "output.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown tool 'unknown'"));
    }

    #[test]
    fn test_download_cmd_with_special_characters() {
        let cmd = download_cmd("curl", "https://example.com/file with spaces.txt", "output file.txt").unwrap();
        assert_eq!(cmd, "\"$CURL_BIN\" -s -o \"output file.txt\" \"https://example.com/file with spaces.txt\"");
    }
}
