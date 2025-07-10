use clap::Parser;
use std;

mod cli;
mod download;
mod config;
mod output;
mod error;
mod http;
mod init;
mod file_utils;
mod fast_download;
mod lazy_config;
mod minimal_http;
mod ultra_fast;
mod native_http;
mod simd_ops;
mod ultimate_fast;
mod multithreaded_download;
mod retry;
mod cookie;

use cli::Args;
use download::download;
use config::Config;
use output::Logger;
use error::{Result, RuGetError};
use init::init_config;
use file_utils::load_urls_from_file;
use fast_download::{fast_single_download, should_use_fast_path};
use lazy_config::{apply_config_if_needed, skip_config_for_simple_download};
use cli::{LogFormat};

fn main() {
    if let Err(e) = run() {
        // Create a basic logger for error reporting
        let logger = Logger::new(false, false);
        logger.error_from_ruget_error(&e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let raw_args = Args::parse();
    
    // Handle --init and exit early (don't create logger yet)
    if raw_args.init {
        let use_json = determine_json_output(&raw_args);
        let logger = Logger::new_with_json(raw_args.quiet, raw_args.verbose, use_json);
        return init_config(&logger);
    }

    let mut args = raw_args;
    
    // Check for fast path BEFORE config loading to avoid config interference
    let can_use_fast_path = should_use_fast_path(&args);
    
    if can_use_fast_path {
        // Fast path: minimal single URL download
        let url = &args.urls[0];
        if let Some(output_path) = args.output.as_deref() {
            return fast_single_download(url, Some(output_path), args.quiet);
        } else {
            return fast_single_download(url, None, args.quiet);
        }
    }
    
    // Regular path: apply config and continue
    if !skip_config_for_simple_download(&args) {
        apply_config_if_needed(&mut args);
    }

    // Load URLs from --input file if provided
    if let Some(ref input_path) = args.input {
        let file_urls = load_urls_from_file(input_path)?;
        args.urls.extend(file_urls);
    }

    if args.urls.is_empty() {
        eprintln!("Error: No URLs provided via --input or CLI.");
        return Err(RuGetError::parse("No URLs provided".into()));
    }

    // Create the logger with JSON option
    let use_json = determine_json_output(&args);
    let logger = Logger::new_with_json(args.quiet, args.verbose, use_json);

    // Fall back to full download functionality
    download(args, &logger)
}

/// Determine whether to use JSON output based on configuration
fn determine_json_output(args: &Args) -> bool {
    // Priority: CLI flag > log_format config > default false
    if args.log_json {
        return true;
    }
    
    if let Some(format) = &args.log_format {
        match format {
            LogFormat::Json => true,
            LogFormat::Text => false,
        }
    } else {
        false
    }
}
