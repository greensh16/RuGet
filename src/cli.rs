use clap::Parser;

/// A simple wget-like tool written in Rust
#[derive(Parser, Debug)]
#[command(name = "ruget", version="0.1.0", about = "A simple downloader")]
pub struct Args {
    /// URL to fetch
    pub url: String,

    /// Output file path (optional)
    #[arg(short, long)]
    pub output: Option<String>,
}