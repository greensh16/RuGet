use clap::{Parser, ArgAction};

/// A simple wget-like tool written in Rust
#[derive(Parser, Debug)]
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

    /// Retry on failure
    #[arg(long, default_value = "3")]
    pub retries: u32,

    /// Verbose output
    #[arg(long, action = ArgAction::SetTrue)]
    pub verbose: bool,

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
}