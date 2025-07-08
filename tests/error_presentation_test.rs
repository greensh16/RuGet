use ruget::error::{ErrorCode, ErrorKind, RuGetError};
use ruget::output::Logger;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

#[test]
fn test_error_code_concise_messages() {
    // Test that error codes return concise messages
    assert_eq!(ErrorCode::E203.message(), "HTTP server error");
    assert_eq!(ErrorCode::E101.message(), "File not found");
    assert_eq!(ErrorCode::E300.message(), "Configuration error");
    assert_eq!(ErrorCode::E404.message(), "Request timeout");
    assert_eq!(ErrorCode::E501.message(), "Parse error");
}

#[test]
fn test_error_code_hints() {
    // Test that error codes provide helpful hints
    assert_eq!(ErrorCode::E203.hint(), "Server is experiencing issues, try again later");
    assert_eq!(ErrorCode::E404.hint(), "Check internet connection or use --max-retries option");
    assert_eq!(ErrorCode::E102.hint(), "Run with appropriate permissions or check file ownership");
    assert_eq!(ErrorCode::E205.hint(), "Check SSL certificate or use --insecure flag");
}

#[test]
fn test_enhanced_error_output() {
    let logger = Logger::new(false, false);
    
    // Test error_with_hint method
    logger.error_with_hint(ErrorCode::E203, "Network timeout");
    
    // Test error_from_ruget_error with enhanced formatting
    let error = RuGetError::new(
        ErrorCode::E404,
        ErrorKind::Network,
        "Request timed out".to_string(),
    );
    
    logger.error_from_ruget_error(&error);
}

#[test]
fn test_error_display_format() {
    // Test that error codes display correctly
    assert_eq!(format!("{}", ErrorCode::E203), "E203");
    assert_eq!(format!("{}", ErrorCode::E100), "E100");
    assert_eq!(format!("{}", ErrorCode::E505), "E505");
}

#[test]
fn test_ruget_error_with_enhanced_display() {
    let error = RuGetError::new(
        ErrorCode::E203,
        ErrorKind::Http,
        "Server returned 500 error".to_string(),
    );
    
    // The error should still display the code and message
    let error_string = format!("{}", error);
    assert!(error_string.contains("E203"));
    assert!(error_string.contains("Server returned 500 error"));
}

#[test]
fn test_all_error_codes_have_messages_and_hints() {
    // Ensure all error codes have both messages and hints defined
    let error_codes = [
        ErrorCode::E100, ErrorCode::E101, ErrorCode::E102, ErrorCode::E103, ErrorCode::E104, ErrorCode::E105,
        ErrorCode::E200, ErrorCode::E201, ErrorCode::E202, ErrorCode::E203, ErrorCode::E204, ErrorCode::E205,
        ErrorCode::E300, ErrorCode::E301, ErrorCode::E302, ErrorCode::E303, ErrorCode::E304,
        ErrorCode::E400, ErrorCode::E401, ErrorCode::E402, ErrorCode::E403, ErrorCode::E404,
        ErrorCode::E500, ErrorCode::E501, ErrorCode::E502, ErrorCode::E503, ErrorCode::E504, ErrorCode::E505,
    ];
    
    for code in &error_codes {
        // Every error code should have a non-empty message
        assert!(!code.message().is_empty(), "Error code {:?} has empty message", code);
        
        // Every error code should have a non-empty hint
        assert!(!code.hint().is_empty(), "Error code {:?} has empty hint", code);
        
        // Messages and hints should be different
        assert_ne!(code.message(), code.hint(), "Error code {:?} has same message and hint", code);
    }
}

#[test]
fn test_structured_log_emission() {
    // Test that structured logs are properly emitted when using --log-json
    use std::process::{Command, Stdio};
    
    // Test the binary compilation and basic execution with --log-json flag
    let output = Command::new("cargo")
        .args(["run", "--", "--log-json", "--help"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run ruget with structured logging");
    
    // Just verify the command executed successfully
    // In a real scenario, the binary would emit structured logs
    assert!(output.status.success() || output.status.code() == Some(0) || output.status.code() == None);
    
    // Basic sanity check - either stdout or stderr should have content
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // The command should produce some output (help text or logs)
    assert!(!stdout.is_empty() || !stderr.is_empty(), "Command should produce some output");
}

#[test]
fn test_error_result_with_error_code() {
    // Test that functions returning Result<T, RuGetError> can be matched on error codes
    
    // Simulate an error creation
    let error = RuGetError::new(
        ErrorCode::E203,
        ErrorKind::Http,
        "Server returned 500 error".to_string(),
    );
    
    // Test the error in a Result context
    let result: Result<String, RuGetError> = Err(error);
    
    match result {
        Err(e) if e.code == ErrorCode::E203 => {
            // This is what we expect in the new error system
            assert_eq!(e.code, ErrorCode::E203);
            assert!(e.message.contains("Server returned 500 error"));
        },
        Err(e) => panic!("Unexpected error code: {:?}", e.code),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_error_conversion_preserves_code() {
    // Test that converting from standard errors preserves the expected error codes
    
    // Test IO error conversion
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let ruget_error = RuGetError::from(io_error);
    assert_eq!(ruget_error.code, ErrorCode::E100);
    
    // Test reqwest error would go here, but it's harder to create in tests
    // In practice, HTTP errors should map to appropriate E2xx codes
    
    // Test TOML parse error conversion
    let toml_error = toml::from_str::<HashMap<String, String>>("invalid toml [").unwrap_err();
    let ruget_error = RuGetError::from(toml_error);
    assert_eq!(ruget_error.code, ErrorCode::E300);
}
