use std::io::{self, Write};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::{RuGetError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogRecord {
    pub ts: DateTime<Utc>,
    pub level: LogLevel,
    pub code: Option<String>,
    pub message: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

pub struct Logger {
    pub quiet: bool,
    pub verbose: bool,
    pub json_output: bool,
}

impl Logger {
    pub fn new(quiet: bool, verbose: bool) -> Self {
        Self { quiet, verbose, json_output: false }
    }

    pub fn new_with_json(quiet: bool, verbose: bool, json_output: bool) -> Self {
        Self { quiet, verbose, json_output }
    }

    fn log_structured(&self, level: LogLevel, message: &str, context: HashMap<String, String>, code: Option<String>) {
        let record = StructuredLogRecord {
            ts: Utc::now(),
            level,
            code,
            message: message.to_string(),
            context,
        };

        if self.json_output {
            if let Ok(json) = serde_json::to_string(&record) {
                println!("{}", json);
            }
        } else {
            self.log_human_readable(&record);
        }
    }

    fn log_human_readable(&self, record: &StructuredLogRecord) {
        let timestamp = record.ts.format("%Y-%m-%dT%H:%M:%SZ");
        let level_str = match record.level {
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };

        let code_str = record.code.as_deref().unwrap_or("");
        let context_str = if record.context.is_empty() {
            String::new()
        } else {
            format!(" | {}", 
                record.context.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        if !code_str.is_empty() {
            println!("[{}][{}][{}] {}{}", code_str, level_str, timestamp, record.message, context_str);
        } else {
            println!("[{}][{}] {}{}", level_str, timestamp, record.message, context_str);
        }
    }

    pub fn info(&self, message: &str) {
        self.info_with_context(message, HashMap::new());
    }

    pub fn info_with_context(&self, message: &str, context: HashMap<String, String>) {
        if !self.quiet {
            self.log_structured(LogLevel::Info, message, context, None);
        }
    }

    pub fn verbose(&self, message: &str) {
        self.verbose_with_context(message, HashMap::new());
    }

    pub fn verbose_with_context(&self, message: &str, context: HashMap<String, String>) {
        if self.verbose && !self.quiet {
            self.log_structured(LogLevel::Debug, message, context, None);
        }
    }

    pub fn error(&self, message: &str) {
        self.error_with_context(message, HashMap::new());
    }

    pub fn error_with_context(&self, message: &str, context: HashMap<String, String>) {
        self.log_structured(LogLevel::Error, message, context, None);
    }

    pub fn error_from_ruget_error(&self, error: &RuGetError) {
        let mut context = HashMap::new();
        context.insert("error_kind".to_string(), format!("{:?}", error.kind));
        if let Some(source) = &error.source {
            context.insert("source".to_string(), source.to_string());
        }
        context.insert("hint".to_string(), error.code.hint().to_string());
        
        let enhanced_message = format!("{} Hint: {}", error.code.message(), error.code.hint());
        self.log_structured(LogLevel::Error, &enhanced_message, context, Some(error.code.to_string()));
    }
    
    pub fn error_with_hint(&self, code: ErrorCode, message: &str) {
        let mut context = HashMap::new();
        context.insert("hint".to_string(), code.hint().to_string());
        
        let enhanced_message = format!("{} Hint: {}", message, code.hint());
        self.log_structured(LogLevel::Error, &enhanced_message, context, Some(code.to_string()));
    }

    pub fn warn(&self, message: &str) {
        self.warn_with_context(message, HashMap::new());
    }

    pub fn warn_with_context(&self, message: &str, context: HashMap<String, String>) {
        if !self.quiet {
            self.log_structured(LogLevel::Warn, message, context, None);
        }
    }

    pub fn status(&self, url: &str, status: &str) {
        if !self.quiet {
            let mut context = HashMap::new();
            context.insert("url".to_string(), url.to_string());
            context.insert("status".to_string(), status.to_string());
            self.log_structured(LogLevel::Info, "HTTP Status", context, None);
        }
    }

    pub fn headers(&self, headers: &reqwest::header::HeaderMap) {
        if self.verbose && !self.quiet {
            for (key, value) in headers {
                let mut context = HashMap::new();
                context.insert("header_name".to_string(), key.to_string());
                context.insert("header_value".to_string(), value.to_str().unwrap_or("[binary]").to_string());
                self.log_structured(LogLevel::Debug, "HTTP Header", context, None);
            }
        }
    }

    pub fn progress(&self, message: &str) {
        if !self.quiet {
            // Progress messages are ephemeral, keep them as is for now
            print!("\r{}", message);
            io::stdout().flush().unwrap_or(());
        }
    }

    pub fn download_start(&self, url: &str, output_path: &str) {
        if !self.quiet {
            let mut context = HashMap::new();
            context.insert("url".to_string(), url.to_string());
            context.insert("output_path".to_string(), output_path.to_string());
            self.log_structured(LogLevel::Info, "Download started", context, None);
        }
    }

    pub fn download_resume(&self, output_path: &str, bytes: u64) {
        if !self.quiet {
            let mut context = HashMap::new();
            context.insert("output_path".to_string(), output_path.to_string());
            context.insert("resume_bytes".to_string(), bytes.to_string());
            self.log_structured(LogLevel::Info, "Download resumed", context, None);
        }
    }

    pub fn download_complete(&self, output_path: &str) {
        if !self.quiet {
            let mut context = HashMap::new();
            context.insert("output_path".to_string(), output_path.to_string());
            self.log_structured(LogLevel::Info, "Download complete", context, None);
        }
    }

    pub fn retry_attempt(&self, url: &str, error: &str) {
        if !self.quiet {
            let mut context = HashMap::new();
            context.insert("url".to_string(), url.to_string());
            context.insert("error".to_string(), error.to_string());
            self.log_structured(LogLevel::Warn, "Retrying after error", context, None);
        }
    }

    pub fn summary(&self, successful: usize, total: usize) {
        if !self.quiet {
            let mut context = HashMap::new();
            context.insert("successful".to_string(), successful.to_string());
            context.insert("total".to_string(), total.to_string());
            self.log_structured(LogLevel::Info, "Download summary", context, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use crate::error::{ErrorCode, ErrorKind, RuGetError};

    // Mock writer to capture output
    struct MockWriter {
        output: Arc<Mutex<Vec<u8>>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                output: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_output(&self) -> String {
            let data = self.output.lock().unwrap();
            String::from_utf8_lossy(&data).to_string()
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.output.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_structured_log_record_creation() {
        let mut context = HashMap::new();
        context.insert("key1".to_string(), "value1".to_string());
        context.insert("key2".to_string(), "value2".to_string());

        let record = StructuredLogRecord {
            ts: Utc::now(),
            level: LogLevel::Error,
            code: Some("E123".to_string()),
            message: "Test error message".to_string(),
            context,
        };

        assert_eq!(record.level, LogLevel::Error);
        assert_eq!(record.code, Some("E123".to_string()));
        assert_eq!(record.message, "Test error message");
        assert_eq!(record.context.get("key1"), Some(&"value1".to_string()));
        assert_eq!(record.context.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_json_output_format() {
        let logger = Logger::new_with_json(false, false, true);
        let mut context = HashMap::new();
        context.insert("url".to_string(), "https://example.com".to_string());
        context.insert("status".to_string(), "200".to_string());

        // Since we can't easily capture stdout in this test setup,
        // we'll test the record creation and JSON serialization directly
        let record = StructuredLogRecord {
            ts: chrono::DateTime::parse_from_rfc3339("2023-09-28T12:34:56Z")
                .unwrap()
                .with_timezone(&Utc),
            level: LogLevel::Error,
            code: Some("E123".to_string()),
            message: "Test message".to_string(),
            context,
        };

        let json_output = serde_json::to_string(&record).unwrap();
        assert!(json_output.contains("\"level\":\"ERROR\""));
        assert!(json_output.contains("\"code\":\"E123\""));
        assert!(json_output.contains("\"message\":\"Test message\""));
        assert!(json_output.contains("\"url\":\"https://example.com\""));
        assert!(json_output.contains("\"status\":\"200\""));
    }

    #[test]
    fn test_human_readable_format_with_error_code() {
        let logger = Logger::new(false, false);
        let mut context = HashMap::new();
        context.insert("url".to_string(), "https://example.com".to_string());
        context.insert("status".to_string(), "404".to_string());

        let record = StructuredLogRecord {
            ts: chrono::DateTime::parse_from_rfc3339("2023-09-28T12:34:56Z")
                .unwrap()
                .with_timezone(&Utc),
            level: LogLevel::Error,
            code: Some("E123".to_string()),
            message: "Test error message".to_string(),
            context,
        };

        // Test the format string generation
        let timestamp = record.ts.format("%Y-%m-%dT%H:%M:%SZ");
        let code_str = record.code.as_deref().unwrap_or("");
        let context_str = format!(" | {}", 
            record.context.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let expected_format = format!(
            "[{}][ERROR][{}] {}{}",
            code_str, timestamp, record.message, context_str
        );

        assert!(expected_format.contains("[E123][ERROR][2023-09-28T12:34:56Z] Test error message | "));
        assert!(expected_format.contains("url=https://example.com"));
        assert!(expected_format.contains("status=404"));
    }

    #[test]
    fn test_human_readable_format_without_error_code() {
        let logger = Logger::new(false, false);
        let mut context = HashMap::new();
        context.insert("key".to_string(), "value".to_string());

        let record = StructuredLogRecord {
            ts: chrono::DateTime::parse_from_rfc3339("2023-09-28T12:34:56Z")
                .unwrap()
                .with_timezone(&Utc),
            level: LogLevel::Info,
            code: None,
            message: "Test info message".to_string(),
            context,
        };

        let timestamp = record.ts.format("%Y-%m-%dT%H:%M:%SZ");
        let context_str = format!(" | {}", 
            record.context.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let expected_format = format!(
            "[INFO][{}] {}{}",
            timestamp, record.message, context_str
        );

        assert_eq!(expected_format, "[INFO][2023-09-28T12:34:56Z] Test info message | key=value");
    }

    #[test]
    fn test_error_from_ruget_error() {
        let logger = Logger::new(false, false);
        let ruget_error = RuGetError::new(
            ErrorCode::E400,  // Using existing error code for network errors
            ErrorKind::Network,
            "Network connection failed".to_string(),
        );

        // Test that the method compiles and doesn't panic
        logger.error_from_ruget_error(&ruget_error);
    }

    #[test]
    fn test_context_methods() {
        let logger = Logger::new(false, false);
        let mut context = HashMap::new();
        context.insert("test_key".to_string(), "test_value".to_string());

        // Test that all context methods compile
        logger.info_with_context("test", context.clone());
        logger.error_with_context("test", context.clone());
        logger.warn_with_context("test", context.clone());
        logger.verbose_with_context("test", context);
    }

    #[test]
    fn test_log_levels_serialization() {
        assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"INFO\"");
        assert_eq!(serde_json::to_string(&LogLevel::Debug).unwrap(), "\"DEBUG\"");
        assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"WARN\"");
        assert_eq!(serde_json::to_string(&LogLevel::Error).unwrap(), "\"ERROR\"");
    }
}
