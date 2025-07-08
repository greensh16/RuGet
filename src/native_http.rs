use crate::error::{Result, RuGetError};
use std::collections::HashMap;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpStream, ToSocketAddrs, IpAddr};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// DNS cache to avoid repeated lookups
static DNS_CACHE: OnceLock<Arc<Mutex<HashMap<String, (Vec<IpAddr>, Instant)>>>> = OnceLock::new();

/// Connection pool for reusing TCP connections
static CONNECTION_POOL: OnceLock<Arc<Mutex<HashMap<String, Vec<(TcpStream, Instant)>>>>> = OnceLock::new();

const DNS_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const CONNECTION_TTL: Duration = Duration::from_secs(60);  // 1 minute

/// Ultra-high performance HTTP client using native TCP sockets
pub struct NativeHttpClient {
    timeout: Duration,
    user_agent: String,
}

impl NativeHttpClient {
    pub fn new() -> Self {
        // Initialize global caches
        DNS_CACHE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        CONNECTION_POOL.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        
        Self {
            timeout: Duration::from_secs(10),
            user_agent: "RuGet/2.0 (Ultra-Fast)".to_string(),
        }
    }

    /// Perform an ultra-fast HTTP GET request
    pub fn get(&self, url: &str) -> Result<Vec<u8>> {
        let parsed = parse_url_fast(url)?;
        
        match parsed.scheme.as_str() {
            "http" => self.get_http(&parsed),
            "https" => self.get_https(&parsed),
            _ => Err(RuGetError::network("Unsupported scheme".into())),
        }
    }

    /// HTTP GET using raw TCP socket
    fn get_http(&self, url: &ParsedUrl) -> Result<Vec<u8>> {
        let port = url.port.unwrap_or(80);
        let host_port = format!("{}:{}", url.host, port);
        
        // Try to get connection from pool first
        let mut stream = if let Some(pooled) = self.get_pooled_connection(&host_port) {
            pooled
        } else {
            self.create_connection(&url.host, port)?
        };
        
        // Send HTTP request with minimal headers
        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             User-Agent: {}\r\n\
             Accept: */*\r\n\
             Connection: keep-alive\r\n\r\n",
            url.path, url.host, self.user_agent
        );
        
        stream.write_all(request.as_bytes())?;
        
        // Read response with optimized parsing
        let response = self.read_http_response(&mut stream)?;
        
        // Return connection to pool if still alive
        self.return_connection_to_pool(host_port, stream);
        
        Ok(response)
    }

    /// HTTPS GET (fallback to optimized reqwest for TLS)
    fn get_https(&self, url: &ParsedUrl) -> Result<Vec<u8>> {
        // For HTTPS, use highly optimized reqwest configuration
        let full_url = format!("https://{}{}", url.host_with_port(), url.path);
        
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .pool_idle_timeout(Duration::from_secs(0))
            .pool_max_idle_per_host(1) // Minimal pooling
            .tcp_nodelay(true)
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;
        
        let response = client
            .get(&full_url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "*/*")
            .send()?;
        
        if !response.status().is_success() {
            return Err(RuGetError::network(format!("HTTP {}", response.status())));
        }
        
        Ok(response.bytes()?.to_vec())
    }

    /// Get cached DNS resolution or perform new lookup
    fn resolve_host(&self, host: &str) -> Result<Vec<IpAddr>> {
        let cache = DNS_CACHE.get().unwrap();
        let mut cache_lock = cache.lock().unwrap();
        
        // Check cache first
        if let Some((ips, timestamp)) = cache_lock.get(host) {
            if timestamp.elapsed() < DNS_CACHE_TTL {
                return Ok(ips.clone());
            }
        }
        
        // Perform DNS lookup
        let addresses: Vec<IpAddr> = format!("{}:0", host)
            .to_socket_addrs()?
            .map(|addr| addr.ip())
            .collect();
        
        if addresses.is_empty() {
            return Err(RuGetError::network(format!("Failed to resolve {}", host)));
        }
        
        // Cache the result
        cache_lock.insert(host.to_string(), (addresses.clone(), Instant::now()));
        
        Ok(addresses)
    }

    /// Create optimized TCP connection
    fn create_connection(&self, host: &str, port: u16) -> Result<TcpStream> {
        let ips = self.resolve_host(host)?;
        
        // Try connecting to each IP address
        for ip in ips {
            let addr = (ip, port);
            if let Ok(stream) = TcpStream::connect_timeout(&addr.into(), self.timeout) {
                // Optimize socket settings
                stream.set_read_timeout(Some(self.timeout))?;
                stream.set_write_timeout(Some(self.timeout))?;
                stream.set_nodelay(true)?; // Disable Nagle's algorithm
                
                return Ok(stream);
            }
        }
        
        Err(RuGetError::network(format!("Failed to connect to {}:{}", host, port)))
    }

    /// Try to get a connection from the pool
    fn get_pooled_connection(&self, host_port: &str) -> Option<TcpStream> {
        let pool = CONNECTION_POOL.get().unwrap();
        let mut pool_lock = pool.lock().unwrap();
        
        if let Some(connections) = pool_lock.get_mut(host_port) {
            // Remove expired connections
            connections.retain(|(_, timestamp)| timestamp.elapsed() < CONNECTION_TTL);
            
            // Return a fresh connection if available
            if let Some((stream, _)) = connections.pop() {
                return Some(stream);
            }
        }
        
        None
    }

    /// Return connection to pool for reuse
    fn return_connection_to_pool(&self, host_port: String, stream: TcpStream) {
        let pool = CONNECTION_POOL.get().unwrap();
        let mut pool_lock = pool.lock().unwrap();
        
        pool_lock
            .entry(host_port)
            .or_insert_with(Vec::new)
            .push((stream, Instant::now()));
    }

    /// Read and parse HTTP response with optimized parsing
    fn read_http_response(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut reader = BufReader::new(stream);
        
        // Read status line
        let mut status_line = String::new();
        reader.read_line(&mut status_line)?;
        
        if !status_line.contains("200") && !status_line.contains("206") {
            return Err(RuGetError::network("HTTP request failed".into()));
        }
        
        // Read headers to find content length
        let mut content_length = None;
        let mut chunked = false;
        
        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line)?;
            
            if header_line.trim().is_empty() {
                break; // End of headers
            }
            
            let header_lower = header_line.to_lowercase();
            if header_lower.starts_with("content-length:") {
                if let Some(len_str) = header_line.split(':').nth(1) {
                    content_length = len_str.trim().parse().ok();
                }
            } else if header_lower.contains("transfer-encoding: chunked") {
                chunked = true;
            }
        }
        
        // Read body
        if chunked {
            self.read_chunked_body(&mut reader)
        } else if let Some(len) = content_length {
            self.read_fixed_body(&mut reader, len)
        } else {
            // Read until connection closes
            let mut body = Vec::new();
            reader.read_to_end(&mut body)?;
            Ok(body)
        }
    }

    /// Read fixed-length body (most efficient)
    fn read_fixed_body(&self, reader: &mut BufReader<&mut TcpStream>, length: usize) -> Result<Vec<u8>> {
        let mut body = vec![0; length];
        reader.read_exact(&mut body)?;
        Ok(body)
    }

    /// Read chunked body
    fn read_chunked_body(&self, reader: &mut BufReader<&mut TcpStream>) -> Result<Vec<u8>> {
        let mut body = Vec::new();
        
        loop {
            let mut chunk_size_line = String::new();
            reader.read_line(&mut chunk_size_line)?;
            
            let chunk_size = usize::from_str_radix(chunk_size_line.trim(), 16)
                .map_err(|_| RuGetError::network("Invalid chunk size".into()))?;
            
            if chunk_size == 0 {
                break; // End of chunks
            }
            
            let mut chunk = vec![0; chunk_size];
            reader.read_exact(&mut chunk)?;
            body.extend_from_slice(&chunk);
            
            // Read trailing CRLF
            let mut trailing = String::new();
            reader.read_line(&mut trailing)?;
        }
        
        Ok(body)
    }
}

impl Default for NativeHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct ParsedUrl {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
}

impl ParsedUrl {
    fn host_with_port(&self) -> String {
        if let Some(port) = self.port {
            format!("{}:{}", self.host, port)
        } else {
            self.host.clone()
        }
    }
}

/// Ultra-fast URL parsing optimized for common cases
fn parse_url_fast(url: &str) -> Result<ParsedUrl> {
    let (scheme, rest) = if url.starts_with("https://") {
        ("https", &url[8..])
    } else if url.starts_with("http://") {
        ("http", &url[7..])
    } else {
        return Err(RuGetError::parse("Unsupported URL scheme".into()));
    };

    let (host_port, path) = if let Some(slash_pos) = rest.find('/') {
        (&rest[..slash_pos], &rest[slash_pos..])
    } else {
        (rest, "/")
    };

    let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
        let host = &host_port[..colon_pos];
        let port_str = &host_port[colon_pos + 1..];
        let port = port_str.parse()
            .map_err(|_| RuGetError::parse("Invalid port".into()))?;
        (host, Some(port))
    } else {
        (host_port, None)
    };

    Ok(ParsedUrl {
        scheme: scheme.to_string(),
        host: host.to_string(),
        port,
        path: path.to_string(),
    })
}

/// Clear caches (useful for testing or memory management)
pub fn clear_caches() {
    if let Some(dns_cache) = DNS_CACHE.get() {
        dns_cache.lock().unwrap().clear();
    }
    if let Some(conn_pool) = CONNECTION_POOL.get() {
        conn_pool.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_fast() {
        let url = parse_url_fast("https://example.com:8080/path/to/file").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/path/to/file");

        let url = parse_url_fast("http://example.com/").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/");
    }

    #[test]
    fn test_host_with_port() {
        let url = ParsedUrl {
            scheme: "https".to_string(),
            host: "example.com".to_string(),
            port: Some(8080),
            path: "/".to_string(),
        };
        assert_eq!(url.host_with_port(), "example.com:8080");

        let url = ParsedUrl {
            scheme: "https".to_string(),
            host: "example.com".to_string(),
            port: None,
            path: "/".to_string(),
        };
        assert_eq!(url.host_with_port(), "example.com");
    }
}
