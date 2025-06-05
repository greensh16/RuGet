use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

mod cli;
mod download;

use cli::Args;
use download::download;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Args::parse();

    // Load URLs from file if provided
    if let Some(ref input_path) = args.input {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let file_urls = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .collect::<Vec<String>>();

        args.urls.extend(file_urls);
    }

    if args.urls.is_empty() {
        eprintln!("Error: No URLs provided (use positional or --input)");
        std::process::exit(1);
    }

    download(args)
}