use crate::error::{Result, RuGetError};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Minimal HTTP client using only std library for maximum performance
pub struct MinimalHttpClient {
    timeout: Duration,
}

impl MinimalHttpClient {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Perform a simple HTTP GET request
    pub fn get(&self, url: &str) -> Result<Vec<u8>> {
        let parsed_url = parse_url(url)?;
        
        // For HTTPS, we still need to use a proper TLS implementation
        // Fall back to reqwest for HTTPS to maintain security
        if parsed_url.scheme == "https" {
            return self.get_with_reqwest(url);
        }
        
        // For HTTP, use raw socket connection
        self.get_http_raw(&parsed_url)
    }

    fn get_http_raw(&self, url: &ParsedUrl) -> Result<Vec<u8>> {
        let port = url.port.unwrap_or(80);
        let addr = format!("{}:{}", url.host, port);
        
        // Connect to server
        let mut stream = TcpStream::connect_timeout(
            &addr.to_socket_addrs()?.next().unwrap(),
            self.timeout
        )?;
        
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        
        // Send HTTP request
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: RuGet/1.0\r\n\r\n",
            url.path, url.host
        );
        
        stream.write_all(request.as_bytes())?;
        
        // Read response
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        
        // Parse HTTP response
        let response_str = String::from_utf8_lossy(&response);
        let parts: Vec<&str> = response_str.splitn(2, "\r\n\r\n").collect();
        
        if parts.len() != 2 {
            return Err(RuGetError::network("Invalid HTTP response".into()));
        }
        
        let headers = parts[0];
        let body = parts[1];
        
        // Check status code
        if !headers.starts_with("HTTP/1.1 200") && !headers.starts_with("HTTP/1.0 200") {
            return Err(RuGetError::network("HTTP request failed".into()));
        }
        
        Ok(body.as_bytes().to_vec())
    }

    fn get_with_reqwest(&self, url: &str) -> Result<Vec<u8>> {
        // Minimal reqwest client for HTTPS
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()?;
            
        let response = client.get(url).send()?;
        
        if !response.status().is_success() {
            return Err(RuGetError::network(format!("HTTP {}", response.status())));
        }
        
        Ok(response.bytes()?.to_vec())
    }
}

impl Default for MinimalHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct ParsedUrl {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
}

fn parse_url(url: &str) -> Result<ParsedUrl> {
    // Simple URL parsing - just enough for basic HTTP/HTTPS
    if url.starts_with("http://") {
        let url = &url[7..]; // Remove "http://"
        parse_http_url(url, "http")
    } else if url.starts_with("https://") {
        let url = &url[8..]; // Remove "https://"
        parse_http_url(url, "https")
    } else {
        Err(RuGetError::parse("Unsupported URL scheme".into()))
    }
}

fn parse_http_url(url: &str, scheme: &str) -> Result<ParsedUrl> {
    let parts: Vec<&str> = url.splitn(2, '/').collect();
    let host_port = parts[0];
    let path = if parts.len() > 1 {
        format!("/{}", parts[1])
    } else {
        "/".to_string()
    };

    let (host, port) = if host_port.contains(':') {
        let hp: Vec<&str> = host_port.splitn(2, ':').collect();
        let port = hp[1].parse::<u16>()
            .map_err(|_| RuGetError::parse("Invalid port number".into()))?;
        (hp[0].to_string(), Some(port))
    } else {
        (host_port.to_string(), None)
    };

    Ok(ParsedUrl {
        scheme: scheme.to_string(),
        host,
        port,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let url = parse_url("http://example.com/path").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path");

        let url = parse_url("https://example.com:8080/").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/");
    }
}
