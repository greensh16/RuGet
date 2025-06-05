use clap::Parser;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::copy;

/// A simple wget-like tool written in Rust
#[derive(Parser, Debug)]
#[command(name = "ruget", version="0.1.0", about = "A simple downloader")]
struct Args {
    /// URL to fetch
    url: String,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client = Client::new();
    let mut response = client.get(&args.url).send()?;
    if !response.status().is_success() {
        eprintln!("Failed to download: {}", response.status());
        std::process::exit(1);
    }

    match args.output {
        Some(path) => {
            let mut dest = File::create(path)?;
            copy(&mut response, &mut dest)?;
        }
        None => {
            let mut stdout = std::io::stdout();
            copy(&mut response, &mut stdout)?;
        }
    }

    Ok(())
}