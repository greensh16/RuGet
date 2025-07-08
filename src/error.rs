use std::fmt;

#[cfg(feature = "context")]
use anyhow::Context;

#[derive(Debug)]
pub struct RuGetError {
    pub code: ErrorCode,
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io,
    Http,
    Config,
    Parse,
    Network,
    FileSystem,
    Authentication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // E1xx: I/O errors
    E100, // General I/O error
    E101, // File not found
    E102, // Permission denied
    E103, // Directory creation failed
    E104, // File write error
    E105, // File read error
    
    // E2xx: HTTP errors
    E200, // General HTTP error
    E201, // Connection timeout
    E202, // HTTP client error (4xx)
    E203, // HTTP server error (5xx)
    E204, // Invalid URL
    E205, // SSL/TLS error
    
    // E3xx: Configuration errors
    E300, // General config error
    E301, // Config file not found
    E302, // Invalid config format
    E303, // Missing required config
    E304, // Invalid config value
    
    // E4xx: Network errors
    E400, // General network error
    E401, // DNS resolution failed
    E402, // Network unreachable
    E403, // Connection refused
    E404, // Request timeout
    
    // E5xx: Internal errors
    E500, // General internal error
    E501, // Parse error
    E502, // Authentication error
    E503, // File system error
    E504, // Data corruption
    E505, // Resource exhausted
}

impl fmt::Display for RuGetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code_num = match self {
            ErrorCode::E100 => 100,
            ErrorCode::E101 => 101,
            ErrorCode::E102 => 102,
            ErrorCode::E103 => 103,
            ErrorCode::E104 => 104,
            ErrorCode::E105 => 105,
            ErrorCode::E200 => 200,
            ErrorCode::E201 => 201,
            ErrorCode::E202 => 202,
            ErrorCode::E203 => 203,
            ErrorCode::E204 => 204,
            ErrorCode::E205 => 205,
            ErrorCode::E300 => 300,
            ErrorCode::E301 => 301,
            ErrorCode::E302 => 302,
            ErrorCode::E303 => 303,
            ErrorCode::E304 => 304,
            ErrorCode::E400 => 400,
            ErrorCode::E401 => 401,
            ErrorCode::E402 => 402,
            ErrorCode::E403 => 403,
            ErrorCode::E404 => 404,
            ErrorCode::E500 => 500,
            ErrorCode::E501 => 501,
            ErrorCode::E502 => 502,
            ErrorCode::E503 => 503,
            ErrorCode::E504 => 504,
            ErrorCode::E505 => 505,
        };
        write!(f, "E{:03}", code_num)
    }
}

impl ErrorCode {
    /// Returns a concise human-readable message for the error code
    pub fn message(&self) -> &'static str {
        match self {
            // E1xx: I/O errors
            ErrorCode::E100 => "General I/O error",
            ErrorCode::E101 => "File not found",
            ErrorCode::E102 => "Permission denied",
            ErrorCode::E103 => "Directory creation failed",
            ErrorCode::E104 => "File write error",
            ErrorCode::E105 => "File read error",
            
            // E2xx: HTTP errors
            ErrorCode::E200 => "HTTP request failed",
            ErrorCode::E201 => "Connection timeout",
            ErrorCode::E202 => "HTTP client error",
            ErrorCode::E203 => "HTTP server error",
            ErrorCode::E204 => "Invalid URL",
            ErrorCode::E205 => "SSL/TLS error",
            
            // E3xx: Configuration errors
            ErrorCode::E300 => "Configuration error",
            ErrorCode::E301 => "Config file not found",
            ErrorCode::E302 => "Invalid config format",
            ErrorCode::E303 => "Missing required config",
            ErrorCode::E304 => "Invalid config value",
            
            // E4xx: Network errors
            ErrorCode::E400 => "Network error",
            ErrorCode::E401 => "DNS resolution failed",
            ErrorCode::E402 => "Network unreachable",
            ErrorCode::E403 => "Connection refused",
            ErrorCode::E404 => "Request timeout",
            
            // E5xx: Internal errors
            ErrorCode::E500 => "Internal error",
            ErrorCode::E501 => "Parse error",
            ErrorCode::E502 => "Authentication error",
            ErrorCode::E503 => "File system error",
            ErrorCode::E504 => "Data corruption",
            ErrorCode::E505 => "Resource exhausted",
        }
    }
    
    /// Returns a troubleshooting hint for the error code
    pub fn hint(&self) -> &'static str {
        match self {
            // E1xx: I/O errors
            ErrorCode::E100 => "Check file permissions and disk space",
            ErrorCode::E101 => "Verify the file path exists",
            ErrorCode::E102 => "Run with appropriate permissions or check file ownership",
            ErrorCode::E103 => "Check parent directory permissions and disk space",
            ErrorCode::E104 => "Ensure sufficient disk space and write permissions",
            ErrorCode::E105 => "Check file exists and has read permissions",
            
            // E2xx: HTTP errors
            ErrorCode::E200 => "Check URL validity and server status",
            ErrorCode::E201 => "Check internet connection or use --timeout option",
            ErrorCode::E202 => "Verify URL and request parameters",
            ErrorCode::E203 => "Server is experiencing issues, try again later",
            ErrorCode::E204 => "Check URL format and protocol",
            ErrorCode::E205 => "Check SSL certificate or use --insecure flag",
            
            // E3xx: Configuration errors
            ErrorCode::E300 => "Check configuration file syntax",
            ErrorCode::E301 => "Create config file or specify path with --config",
            ErrorCode::E302 => "Validate TOML syntax in config file",
            ErrorCode::E303 => "Add required configuration values",
            ErrorCode::E304 => "Check config value format and constraints",
            
            // E4xx: Network errors
            ErrorCode::E400 => "Check internet connection and network settings",
            ErrorCode::E401 => "Check DNS settings or use IP address",
            ErrorCode::E402 => "Check network connectivity and routing",
            ErrorCode::E403 => "Check if service is running and accessible",
            ErrorCode::E404 => "Check internet connection or use --max-retries option",
            
            // E5xx: Internal errors
            ErrorCode::E500 => "Report this issue with debug information",
            ErrorCode::E501 => "Check input format and syntax",
            ErrorCode::E502 => "Check credentials and authentication method",
            ErrorCode::E503 => "Check file system permissions and disk space",
            ErrorCode::E504 => "Verify file integrity and re-download if needed",
            ErrorCode::E505 => "Free up system resources or increase limits",
        }
    }
}

impl std::error::Error for RuGetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|boxed| &**boxed as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for RuGetError {
    fn from(err: std::io::Error) -> Self {
        RuGetError {
            code: ErrorCode::E100,
            kind: ErrorKind::Io,
            message: format!("I/O error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<reqwest::Error> for RuGetError {
    fn from(err: reqwest::Error) -> Self {
        RuGetError {
            code: ErrorCode::E200,
            kind: ErrorKind::Http,
            message: format!("HTTP error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<toml::de::Error> for RuGetError {
    fn from(err: toml::de::Error) -> Self {
        RuGetError {
            code: ErrorCode::E300,
            kind: ErrorKind::Config,
            message: format!("Configuration error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<reqwest::header::InvalidHeaderValue> for RuGetError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        RuGetError {
            code: ErrorCode::E501,
            kind: ErrorKind::Parse,
            message: format!("Invalid header value: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::env::VarError> for RuGetError {
    fn from(err: std::env::VarError) -> Self {
        RuGetError {
            code: ErrorCode::E300,
            kind: ErrorKind::Config,
            message: format!("Environment variable error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<url::ParseError> for RuGetError {
    fn from(err: url::ParseError) -> Self {
        RuGetError {
            code: ErrorCode::E501,
            kind: ErrorKind::Parse,
            message: format!("URL parse error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Helper methods for creating errors
impl RuGetError {
    pub fn new(code: ErrorCode, kind: ErrorKind, message: String) -> Self {
        RuGetError {
            code,
            kind,
            message,
            source: None,
        }
    }
    
    pub fn with_source(code: ErrorCode, kind: ErrorKind, message: String, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        RuGetError {
            code,
            kind,
            message,
            source: Some(source),
        }
    }
    
    // Backward compatibility methods
    pub fn io(err: std::io::Error) -> Self {
        Self::from(err)
    }
    
    pub fn http(err: reqwest::Error) -> Self {
        Self::from(err)
    }
    
    pub fn config(msg: String) -> Self {
        Self::new(ErrorCode::E300, ErrorKind::Config, format!("Configuration error: {}", msg))
    }
    
    pub fn parse(msg: String) -> Self {
        Self::new(ErrorCode::E501, ErrorKind::Parse, format!("Parse error: {}", msg))
    }
    
    pub fn network(msg: String) -> Self {
        Self::new(ErrorCode::E400, ErrorKind::Network, format!("Network error: {}", msg))
    }
    
    pub fn file_system(msg: String) -> Self {
        Self::new(ErrorCode::E503, ErrorKind::FileSystem, format!("File system error: {}", msg))
    }
    
    pub fn authentication(msg: String) -> Self {
        Self::new(ErrorCode::E502, ErrorKind::Authentication, format!("Authentication error: {}", msg))
    }
    
    /// Add context to an existing error
    pub fn with_context(self, context: &str) -> Self {
        RuGetError {
            code: self.code,
            kind: self.kind,
            message: format!("{}: {}", context, self.message),
            source: self.source,
        }
    }
}

// Additional From implementations for string-based errors
impl From<String> for RuGetError {
    fn from(msg: String) -> Self {
        Self::new(ErrorCode::E500, ErrorKind::Parse, msg)
    }
}

impl From<&str> for RuGetError {
    fn from(msg: &str) -> Self {
        Self::new(ErrorCode::E500, ErrorKind::Parse, msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RuGetError>;

// =============================================================================
// BACKWARD COMPATIBILITY LAYER
// =============================================================================

/// Backward compatibility alias for the main error type
/// 
/// **DEPRECATED**: This alias will be removed in v1.0.0.
/// Use `RuGetError` directly instead.
#[deprecated(
    since = "0.2.0",
    note = "Use `RuGetError` directly instead. This alias will be removed in v1.0.0."
)]
pub type RuGetErrorLegacy = RuGetError;

/// Backward compatibility alias for ErrorCode
/// 
/// **DEPRECATED**: This alias will be removed in v1.0.0.
/// Use `ErrorCode` directly instead.
#[deprecated(
    since = "0.2.0",
    note = "Use `ErrorCode` directly instead. This alias will be removed in v1.0.0."
)]
pub type ErrorCodeLegacy = ErrorCode;

/// Backward compatibility alias for ErrorKind
/// 
/// **DEPRECATED**: This alias will be removed in v1.0.0.
/// Use `ErrorKind` directly instead.
#[deprecated(
    since = "0.2.0",
    note = "Use `ErrorKind` directly instead. This alias will be removed in v1.0.0."
)]
pub type ErrorKindLegacy = ErrorKind;

// Legacy error creation functions (deprecated)
impl RuGetError {
    /// Create a legacy-style error
    /// 
    /// **DEPRECATED**: Use `RuGetError::new()` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `RuGetError::new()` instead. This function will be removed in v1.0.0."
    )]
    pub fn legacy_new(message: String) -> Self {
        Self::new(ErrorCode::E500, ErrorKind::Parse, message)
    }
    
    /// Create a legacy-style I/O error
    /// 
    /// **DEPRECATED**: Use `RuGetError::from(io_error)` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `RuGetError::from(io_error)` instead. This function will be removed in v1.0.0."
    )]
    pub fn legacy_io_error(message: String) -> Self {
        Self::new(ErrorCode::E100, ErrorKind::Io, message)
    }
    
    /// Create a legacy-style HTTP error
    /// 
    /// **DEPRECATED**: Use `RuGetError::from(http_error)` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `RuGetError::from(http_error)` instead. This function will be removed in v1.0.0."
    )]
    pub fn legacy_http_error(message: String) -> Self {
        Self::new(ErrorCode::E200, ErrorKind::Http, message)
    }
}

/// Trait for adding context to errors
pub trait WithContext<T> {
    /// Add context to the error
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

/// Implementation for Result<T, E> where E can be converted to RuGetError
impl<T, E> WithContext<T> for std::result::Result<T, E>
where
    E: Into<RuGetError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.into().with_context(&f()))
    }
}

#[cfg(feature = "context")]
mod context_ext {
    use super::*;
    
    /// Extension trait for using anyhow-style context when the feature is enabled
    pub trait AnyhowContextExt<T> {
        /// Add context using anyhow's context method
        fn context_anyhow(self, context: &str) -> Result<T>;
    }
    
    impl<T, E> AnyhowContextExt<T> for std::result::Result<T, E>
    where
        E: Into<RuGetError>,
    {
        fn context_anyhow(self, context: &str) -> Result<T> {
            self.map_err(|e| e.into().with_context(context))
        }
    }
}

#[cfg(feature = "context")]
pub use context_ext::AnyhowContextExt;
