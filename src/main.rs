use clap::Parser;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::{copy, stdout};
use std::process::exit;

mod cli;
use cli::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create HTTP client
    let client = Client::new();
    let response = client.get(&args.url).send()?;

    // Print HTTP status
    let status = response.status();
    println!("Status: {}", status);

    // Print headers
    println!("Headers:");
    for (key, value) in response.headers() {
        println!("  {}: {}", key, value.to_str().unwrap_or("[binary]"));
    }

    if !status.is_success() {
        eprintln!("Download failed with status code: {}", status);
        exit(1);
    }

    // Write response body
    let body = response.bytes()?;
    let mut content = body.as_ref();

    match args.output {
        Some(ref path) => {
            let mut file = File::create(path)?;
            copy(&mut content, &mut file)?;
            println!("Saved to {}", path);
        }
        None => {
            let mut out = stdout();
            copy(&mut content, &mut out)?;
        }
    }

    Ok(())
}