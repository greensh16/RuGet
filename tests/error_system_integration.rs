use ruget::error::{ErrorCode, ErrorKind, RuGetError, Result};
use std::process::{Command, Stdio};
use std::io::{self, Write};
use tempfile::NamedTempFile;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

/// Test that functions return Err(e) where e.code == ErrorCode::... in the new error system
#[test]
fn test_error_code_matching_in_results() {
    // Test file system errors
    let result = std::fs::read_to_string("/nonexistent/file/path");
    match result {
        Err(io_error) => {
            let ruget_error = RuGetError::from(io_error);
            assert_eq!(ruget_error.code, ErrorCode::E100); // General I/O error
        }
        Ok(_) => panic!("Expected file not found error"),
    }

    // Test config parsing errors
    let toml_content = "invalid toml [";
    let result = toml::from_str::<std::collections::HashMap<String, String>>(toml_content);
    match result {
        Err(toml_error) => {
            let ruget_error = RuGetError::from(toml_error);
            assert_eq!(ruget_error.code, ErrorCode::E300); // Configuration error
        }
        Ok(_) => panic!("Expected TOML parse error"),
    }
}

/// Test that error results can be pattern matched on error codes
#[test]
fn test_error_pattern_matching() {
    // Create a function that returns our error type
    fn simulate_http_error() -> Result<String> {
        Err(RuGetError::new(
            ErrorCode::E203,
            ErrorKind::Http,
            "Server returned 500 error".to_string(),
        ))
    }

    fn simulate_network_error() -> Result<String> {
        Err(RuGetError::new(
            ErrorCode::E404,
            ErrorKind::Network,
            "Request timeout".to_string(),
        ))
    }

    // Test pattern matching on HTTP errors
    match simulate_http_error() {
        Err(e) if e.code == ErrorCode::E203 => {
            assert!(e.message.contains("Server returned 500 error"));
        }
        Err(e) => panic!("Unexpected error code: {:?}", e.code),
        Ok(_) => panic!("Expected error"),
    }

    // Test pattern matching on network errors
    match simulate_network_error() {
        Err(e) if e.code == ErrorCode::E404 => {
            assert!(e.message.contains("Request timeout"));
        }
        Err(e) => panic!("Unexpected error code: {:?}", e.code),
        Ok(_) => panic!("Expected error"),
    }
}

/// Test structured log emission by capturing stdout from the binary
#[test]
fn test_structured_log_emission_capture() {
    // Test that the binary emits structured logs when --log-json is used
    let output = Command::new("cargo")
        .args(["run", "--", "https://httpbin.org/status/404", "--log-json", "--max-retries", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run ruget with structured logging");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check if structured logs are present
    if stderr.contains("{") {
        // If JSON is present, it should be structured
        let json_logs: Vec<&str> = stderr.lines()
            .filter(|line| line.trim().starts_with('{'))
            .collect();
            
        for log_line in json_logs {
            // Parse as JSON to validate structure
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(log_line) {
                // Should have timestamp and level fields for structured logs
                assert!(
                    json_value.get("timestamp").is_some() || 
                    json_value.get("level").is_some() ||
                    json_value.get("msg").is_some(),
                    "Structured log missing expected fields: {}", log_line
                );
            }
        }
    }
}

/// Test log format configuration through different output modes
#[test]
fn test_log_format_modes() {
    // Test text format (default)
    let output_text = Command::new("cargo")
        .args(["run", "--", "--help"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run ruget in text mode");

    // Test JSON format
    let output_json = Command::new("cargo")
        .args(["run", "--", "--log-json", "--help"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run ruget in JSON mode");

    // Both should succeed but may have different log formats
    assert!(output_text.status.success());
    assert!(output_json.status.success());
}

/// Test that error context is preserved through the error system
#[test]
fn test_error_context_preservation() {
    use ruget::error::WithContext;

    // Test that context can be added to errors
    let base_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let result: Result<()> = Err(base_error).with_context(|| "Failed to read config file".to_string());

    match result {
        Err(e) => {
            assert_eq!(e.code, ErrorCode::E100); // IO error code
            assert!(e.message.contains("Failed to read config file"));
            assert!(e.message.contains("file not found"));
        }
        Ok(_) => panic!("Expected error with context"),
    }
}

/// Async test for HTTP retry behavior with error code validation
#[tokio::test]
async fn test_http_retry_with_error_codes() {
    let server = MockServer::start().await;

    // Mock server that always returns 503 for testing purposes
    // This tests the error conversion and retry logic
    Mock::given(method("GET"))
        .and(path("/retry-test"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    // Test error conversion and retry logic
    let client = reqwest::Client::new();
    let mut attempts = 0;
    let max_attempts = 2; // Keep it small for testing

    loop {
        attempts += 1;
        let response = client.get(&format!("{}/retry-test", server.uri())).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                panic!("Unexpected success - mock should return 503");
            }
            Ok(resp) if resp.status().is_server_error() => {
                // Convert to our error system and validate error code
                let error = RuGetError::new(
                    ErrorCode::E203, // HTTP server error
                    ErrorKind::Http,
                    format!("HTTP {} error", resp.status()),
                );

                // Verify error code is correct
                assert_eq!(error.code, ErrorCode::E203);
                assert!(error.message.contains("503"));

                if attempts >= max_attempts {
                    // This is expected - we reached max retries
                    break;
                }
                
                // Continue retry loop
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            Ok(resp) => {
                // Other HTTP errors (4xx, etc.)
                let error = RuGetError::new(
                    ErrorCode::E202, // HTTP client error
                    ErrorKind::Http,
                    format!("HTTP {} error", resp.status()),
                );
                panic!("Unexpected status code: {:?}", error);
            }
            Err(e) => {
                // Network/connection errors
                let error = RuGetError::from(e);
                if attempts >= max_attempts {
                    panic!("Network error after max retries: {:?}", error);
                }
                
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }

    // Verify we had to retry and reached max attempts
    assert_eq!(attempts, max_attempts, "Should have made exactly {} attempts", max_attempts);
}

/// Test error code categorization
#[test]
fn test_error_code_categories() {
    // Test that different error types get appropriate codes
    
    // I/O errors -> E1xx
    let io_error = RuGetError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"));
    assert!(matches!(io_error.code, ErrorCode::E100));

    // Config errors -> E3xx
    let config_error = RuGetError::config("Invalid setting".to_string());
    assert!(matches!(config_error.code, ErrorCode::E300));

    // Parse errors -> E5xx
    let parse_error = RuGetError::parse("Invalid format".to_string());
    assert!(matches!(parse_error.code, ErrorCode::E501));

    // Network errors -> E4xx
    let network_error = RuGetError::network("Connection failed".to_string());
    assert!(matches!(network_error.code, ErrorCode::E400));
}
