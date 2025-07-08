use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use netrc::Netrc;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use std::{
    fs::File,
    io::BufReader,
};
use crate::error::{Result, RuGetError, WithContext};
use crate::output::Logger;

#[cfg(feature = "context")]
use crate::error::AnyhowContextExt;

/// Build HTTP headers from command line arguments
pub fn build_headers(header_args: &[String], logger: &Logger) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for h in header_args {
        if let Some((k, v)) = h.split_once(':') {
            let name = match k.trim().parse::<HeaderName>() {
                Ok(name) => name,
                Err(_) => {
                    logger.warn(&format!("Invalid header name: {}", k.trim()));
                    continue;
                }
            };
            let value = match v.trim().parse::<HeaderValue>() {
                Ok(value) => value,
                Err(_) => {
                    logger.warn(&format!("Invalid header value: {}", v.trim()));
                    continue;
                }
            };
            headers.insert(name, value);
        }
    }
    headers
}

/// Add netrc authentication to headers if available
pub fn add_netrc_auth(headers: &mut HeaderMap, url: &str) -> Result<()> {
    let parsed_url = reqwest::Url::parse(url)
        .with_context(|| format!("parsing URL for netrc auth: {}", url))?;
    
    if let Some(host) = parsed_url.host_str() {
        let home = std::env::var("HOME")
            .with_context(|| "reading HOME environment variable for netrc auth".to_string())?;
        
        let netrc_path = format!("{}/.netrc", home);
        
        if let Ok(file) = File::open(&netrc_path) {
            if let Ok(netrc) = Netrc::parse(BufReader::new(file)) {
                if let Some((_, machine)) = netrc.hosts.iter().find(|(h, _)| h == host) {
                    if !machine.login.is_empty() {
                        if let Some(password) = &machine.password {
                            if !password.is_empty() {
                                let encoded = BASE64_STANDARD.encode(format!(
                                    "{}:{}",
                                    machine.login, password
                                ));
                                let auth_value = format!("Basic {}", encoded);
                                headers.insert(AUTHORIZATION, auth_value.parse()
                                     .with_context(|| format!("creating auth header for host {}", host))?);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Extract filename from Content-Disposition header
pub fn extract_filename_from_disposition(header: Option<&HeaderValue>) -> Option<String> {
    if let Some(value) = header {
        if let Ok(value_str) = value.to_str() {
            let re = Regex::new(r#"filename="?([^"]+)"?"#).ok()?;
            if let Some(cap) = re.captures(value_str) {
                return Some(cap[1].to_string());
            }
        }
    }
    None
}

/// Get fallback filename from URL
pub fn get_fallback_filename(url: &str) -> String {
    url.split('/')
        .last()
        .filter(|s| !s.is_empty())
        .unwrap_or("download.bin")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_filename() {
        assert_eq!(get_fallback_filename("https://example.com/file.txt"), "file.txt");
        assert_eq!(get_fallback_filename("https://example.com/"), "download.bin");
        assert_eq!(get_fallback_filename("https://example.com"), "download.bin");
    }

    #[test]
    fn test_extract_filename_from_disposition() {
        use reqwest::header::HeaderValue;
        
        let header = HeaderValue::from_static(r#"attachment; filename="test.txt""#);
        assert_eq!(extract_filename_from_disposition(Some(&header)), Some("test.txt".to_string()));
        
        let header = HeaderValue::from_static(r#"attachment; filename=test.txt"#);
        assert_eq!(extract_filename_from_disposition(Some(&header)), Some("test.txt".to_string()));
    }
}
