use clap::{Parser, ArgAction, ValueEnum};

/// Log output format options
#[derive(Clone, Debug, ValueEnum)]
pub enum LogFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
}

/// Log level options
#[derive(Clone, Debug, ValueEnum)]
pub enum LogLevel {
    /// Debug level (most verbose)
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level (least verbose)
    Error,
}

/// A simple wget-like tool written in Rust
#[derive(Parser, Debug, Clone)]
#[command(name = "ruget", version="0.1.5", about = "A simple downloader")]
pub struct Args {
    /// One or more URLs to fetch
    pub urls: Vec<String>,

    /// Optional file with a list of URLs
    #[arg(short, long)]
    pub input: Option<String>,

    /// Output file path (only valid for single URL)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Custom headers
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Resume if file already exists
    #[arg(long)]
    pub resume: bool,

    /// Maximum number of retries on failure
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// Verbose output
    #[arg(long, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Output logs in JSON format
    #[arg(long, action = ArgAction::SetTrue)]
    pub log_json: bool,

    /// Log output format (json or text)
    #[arg(long, value_enum, help = "Log output format")]
    pub log_format: Option<LogFormat>,

    /// Log level (debug, info, warn, error)
    #[arg(long, value_enum, help = "Log level")]
    pub log_level: Option<LogLevel>,

    /// Quiet mode
    #[arg(long, action = ArgAction::SetTrue)]
    pub quiet: bool,

    /// Directory to save downloaded files (used with multiple URLs or --input)
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Number of parallel downloads (default: number of CPUs)
    #[arg(long, default_value = "0")]
    pub jobs: usize,

    /// Path to log file for failed downloads
    #[arg(long, default_value = "rustget_failures.log")]
    pub log: String,

    /// Create a default ~/.rugetrc config template
    #[arg(long)]
    pub init: bool,

    /// Base delay for exponential backoff in milliseconds
    #[arg(long, default_value = "100")]
    pub backoff_base_ms: u64,

    /// Maximum delay for exponential backoff in milliseconds
    #[arg(long, default_value = "60000")]
    pub backoff_max_ms: u64,
}