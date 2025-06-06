use clap::Parser;
use std;

mod cli;
mod download;
mod config;

use cli::Args;
use download::download;
use config::Config;

fn main() {
    let raw_args = Args::parse();

    // Handle --init and exit early
    if raw_args.init {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = format!("{}/.rugetrc", home);
        let content = r#"# ~/.rugetrc

# Default retry count
retries = 3

# Resume partial downloads if possible
resume = true

# Suppress output
quiet = false

# Verbose output (headers, etc)
verbose = false

# Number of parallel jobs (0 = auto)
jobs = 0

# Output directory for downloads
output_dir = "/path/to/save"

# Custom headers to send
headers = [
  "User-Agent: RuGet/1.0",
  "Accept: */*"
]

# Log file path for failed downloads
log = "/tmp/ruget_failures.log"
"#;

        if std::fs::write(&path, content).is_ok() {
            println!("Wrote template to {}", path);
        } else {
            eprintln!("Failed to write to {}", path);
        }

        std::process::exit(0);
    }

    // Load ~/.rugetrc
    let config = Config::from_file();

    // Override raw_args with config values if not set
    let mut args = raw_args;

    if args.retries == 0 {
        args.retries = config.retries.unwrap_or(0) as u32;
    }
    if !args.resume {
        args.resume = config.resume.unwrap_or(false);
    }
    if !args.quiet {
        args.quiet = config.quiet.unwrap_or(false);
    }
    if !args.verbose {
        args.verbose = config.verbose.unwrap_or(false);
    }
    if args.jobs == 0 {
        args.jobs = config.jobs.unwrap_or(0);
    }
    if args.output_dir.is_none() {
        args.output_dir = config.output_dir;
    }
    if args.headers.is_empty() {
        args.headers = config.headers.unwrap_or_default();
    }
    if args.log.is_empty() {
        args.log = config
            .log
            .unwrap_or_else(|| "ruget_failures.log".to_string());
    }

    // Load URLs from --input file if provided
    if let Some(ref input_path) = args.input {
        match std::fs::read_to_string(input_path) {
            Ok(contents) => {
                let file_urls: Vec<String> = contents
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(str::to_string)
                    .collect();

                args.urls.extend(file_urls);
            }
            Err(err) => {
                eprintln!("Failed to read input file '{}': {}", input_path, err);
                std::process::exit(1);
            }
        }
    }

    if args.urls.is_empty() {
        eprintln!("No URLs provided via --input or CLI.");
        std::process::exit(1);
    }

    if let Err(e) = download(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}