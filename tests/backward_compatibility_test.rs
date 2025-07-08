use ruget::{RuGetError, ErrorCode, ErrorKind};

#[allow(deprecated)]
use ruget::{RuGetErrorLegacy, ErrorCodeLegacy, ErrorKindLegacy};

#[test]
fn test_backward_compatibility_aliases() {
    // Test that the deprecated aliases still work
    let _error_legacy: RuGetErrorLegacy = RuGetError::new(
        ErrorCode::E100,
        ErrorKind::Io,
        "Test error".to_string(),
    );
    
    let _code_legacy: ErrorCodeLegacy = ErrorCode::E200;
    let _kind_legacy: ErrorKindLegacy = ErrorKind::Http;
    
    // Test that they are the same type
    assert_eq!(std::mem::size_of::<RuGetError>(), std::mem::size_of::<RuGetErrorLegacy>());
    assert_eq!(std::mem::size_of::<ErrorCode>(), std::mem::size_of::<ErrorCodeLegacy>());
    assert_eq!(std::mem::size_of::<ErrorKind>(), std::mem::size_of::<ErrorKindLegacy>());
}

#[test]
#[allow(deprecated)]
fn test_deprecated_legacy_methods() {
    // Test deprecated methods still work
    let error1 = RuGetError::legacy_new("test".to_string());
    assert_eq!(error1.code, ErrorCode::E500);
    assert_eq!(error1.message, "test");
    
    let error2 = RuGetError::legacy_io_error("io test".to_string());
    assert_eq!(error2.code, ErrorCode::E100);
    assert_eq!(error2.message, "io test");
    
    let error3 = RuGetError::legacy_http_error("http test".to_string());
    assert_eq!(error3.code, ErrorCode::E200);
    assert_eq!(error3.message, "http test");
}

#[test]
fn test_new_error_system() {
    // Test that the new error system works as expected
    let error = RuGetError::new(
        ErrorCode::E301,
        ErrorKind::Config,
        "Config file not found".to_string(),
    );
    
    assert_eq!(error.code, ErrorCode::E301);
    assert_eq!(error.message, "Config file not found");
    assert_eq!(format!("{}", error.code), "E301");
    
    // Test convenience methods
    let config_error = RuGetError::config("test config".to_string());
    assert_eq!(config_error.code, ErrorCode::E300);
    assert_eq!(config_error.message, "Configuration error: test config");
    
    let network_error = RuGetError::network("test network".to_string());
    assert_eq!(network_error.code, ErrorCode::E400);
    assert_eq!(network_error.message, "Network error: test network");
}

#[test]
fn test_error_context() {
    let base_error = RuGetError::new(
        ErrorCode::E100,
        ErrorKind::Io,
        "Base error".to_string(),
    );
    
    let contextual_error = base_error.with_context("While processing file");
    assert_eq!(contextual_error.message, "While processing file: Base error");
    assert_eq!(contextual_error.code, ErrorCode::E100);
}
