pub mod cli;
pub mod download;
pub mod config;
pub mod output;
pub mod error;
pub mod http;
pub mod init;
pub mod file_utils;
pub mod fast_download;
pub mod lazy_config;
pub mod minimal_http;
pub mod ultra_fast;
pub mod native_http;
pub mod simd_ops;
pub mod ultimate_fast;
pub mod multithreaded_download;
pub mod retry;
pub mod cookie;

// Re-export commonly used types for convenience
pub use cli::Args;
pub use error::{Result, RuGetError, ErrorCode, ErrorKind};
pub use retry::{BackoffPolicy, is_transient};

// Deprecated backward compatibility re-exports
#[allow(deprecated)]
pub use error::{RuGetErrorLegacy, ErrorCodeLegacy, ErrorKindLegacy};
