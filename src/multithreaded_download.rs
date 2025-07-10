use crate::cli::Args;
use crate::error::{Result, RuGetError, WithContext};
use crate::http::{build_headers, add_netrc_auth};
use crate::output::Logger;
use crate::retry::{BackoffPolicy, is_transient};
use indicatif::{ProgressBar};
use reqwest::{
    blocking::Client,
    header::RANGE,
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write, Seek, SeekFrom},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use rayon::prelude::*;

#[cfg(feature = "context")]
use crate::error::AnyhowContextExt;

/// Represents a chunk of a file to be downloaded
#[derive(Debug, Clone)]
pub struct DownloadChunk {
    pub start_byte: u64,
    pub end_byte: u64,
    pub chunk_id: usize,
}

/// Downloads a specific chunk of a file
pub fn download_chunk(
    client: &Client,
    url: &str,
    chunk: &DownloadChunk,
    temp_file_path: &str,
    args: &Args,
    pb: Option<Arc<ProgressBar>>,
    logger: &Logger,
) -> Result<()> {
    let mut headers = build_headers(&args.headers, logger);
    add_netrc_auth(&mut headers, url).with_context(|| format!("adding netrc auth for {}", url))?;
    
    // Add range header for this specific chunk
    headers.insert(
        RANGE,
        format!("bytes={}-{}", chunk.start_byte, chunk.end_byte)
            .parse()
            .with_context(|| format!("creating range header for chunk {}", chunk.chunk_id))?
    );

    // Create backoff policy from args
    let backoff_policy = BackoffPolicy::with_params(
        Duration::from_millis(args.backoff_base_ms),
        2.0,
        Duration::from_millis(args.backoff_max_ms),
        true,
    );

    let mut attempt = 0;
    loop {
        let response = client.get(url).headers(headers.clone()).send()
            .with_context(|| format!("sending GET request for chunk {} of {}", chunk.chunk_id, url));

        match response {
            Ok(mut resp) => {
                let status = resp.status();
                if !status.is_success() && status.as_u16() != 206 {
                    // Treat HTTP status errors as retryable
                    attempt += 1;
                    if attempt > args.max_retries {
                        return Err(RuGetError::network(format!(
                            "{}: chunk {} failed with HTTP {} after {} retries", 
                            url, chunk.chunk_id, status, args.max_retries
                        )));
                    }
                    
                    logger.retry_attempt(url, &format!("chunk {} HTTP {}", chunk.chunk_id, status));
                    let delay = backoff_policy.next_delay(attempt - 1);
                    thread::sleep(delay);
                    continue;
                }

                // Create a temporary file for this chunk
                let chunk_temp_path = format!("{}.chunk.{}", temp_file_path, chunk.chunk_id);
                let mut file = File::create(&chunk_temp_path)
                    .with_context(|| format!("creating temporary chunk file {}", chunk_temp_path))?;

                let mut buffer = [0u8; 64 * 1024]; // 64KB buffer for better performance
                let mut bytes_written = 0u64;
                
                loop {
                    let n = resp.read(&mut buffer)
                        .with_context(|| format!("reading response data for chunk {} from {}", chunk.chunk_id, url))?;
                    if n == 0 {
                        break;
                    }
                    
                    file.write_all(&buffer[..n])
                        .with_context(|| format!("writing chunk {} data to {}", chunk.chunk_id, chunk_temp_path))?;
                    
                    bytes_written += n as u64;
                    
                    if let Some(pb) = &pb {
                        pb.inc(n as u64);
                    }
                }

                logger.info(&format!(
                    "Chunk {} ({}-{}) downloaded successfully, {} bytes written",
                    chunk.chunk_id, chunk.start_byte, chunk.end_byte, bytes_written
                ));

                return Ok(());
            }
            Err(e) => {
                attempt += 1;
                if attempt > args.max_retries {
                    let error: RuGetError = e.into();
                    return Err(error.with_context(&format!(
                        "downloading chunk {} of {} after {} retries", 
                        chunk.chunk_id, url, args.max_retries
                    )));
                }

                // Check if error is transient - for now, retry all errors
                // TODO: When we have actual reqwest::Error, check transience properly
                // if let Some(reqwest_error) = e.source().and_then(|s| s.downcast_ref::<reqwest::Error>()) {
                //     if !is_transient(reqwest_error) {
                //         let error: RuGetError = e.into();
                //         return Err(error.with_context(&format!(
                //             "downloading chunk {} of {} - non-transient error", 
                //             chunk.chunk_id, url
                //         )));
                //     }
                // }

                logger.retry_attempt(url, &e.to_string());
                let delay = backoff_policy.next_delay(attempt - 1);
                thread::sleep(delay);
            }
        }
    }
}

/// Combines downloaded chunks into a single file
pub fn combine_chunks(
    output_path: &str,
    temp_file_path: &str,
    num_chunks: usize,
    logger: &Logger,
) -> Result<()> {
    let mut output_file = File::create(output_path)
        .with_context(|| format!("creating final output file {}", output_path))?;

    for chunk_id in 0..num_chunks {
        let chunk_temp_path = format!("{}.chunk.{}", temp_file_path, chunk_id);
        
        if !Path::new(&chunk_temp_path).exists() {
            return Err(RuGetError::file_system(format!(
                "Chunk file {} not found", chunk_temp_path
            )));
        }

        let mut chunk_file = File::open(&chunk_temp_path)
            .with_context(|| format!("opening chunk file {}", chunk_temp_path))?;

        std::io::copy(&mut chunk_file, &mut output_file)
            .with_context(|| format!("copying chunk {} to final file", chunk_id))?;

        // Clean up temporary chunk file
        std::fs::remove_file(&chunk_temp_path)
            .with_context(|| format!("removing temporary chunk file {}", chunk_temp_path))?;
        
        logger.info(&format!("Combined chunk {} into final file", chunk_id));
    }

    Ok(())
}

/// Downloads a file using multiple threads
pub fn multithreaded_download_url(
    client: &Client,
    url: &str,
    output_path: &str,
    args: &Args,
    pb: Option<Arc<ProgressBar>>,
    logger: &Logger,
) -> Result<()> {
    // Get file size first
    let head_response = client.head(url).send()
        .with_context(|| format!("fetching file info for {}", url))?;
    
    let content_length = head_response.content_length()
        .ok_or_else(|| RuGetError::network("Server did not provide content length".to_string()))?;

    // Check if server supports range requests
    let accepts_ranges = head_response
        .headers()
        .get("accept-ranges")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");

    if accepts_ranges != "bytes" && args.jobs > 1 {
        logger.warn("Server does not support range requests, falling back to single-threaded download");
        // Fall back to single-threaded download
        return single_threaded_download(client, url, output_path, args, pb, logger);
    }

    if content_length < 1024 * 1024 || args.jobs <= 1 {
        // For small files or single thread requested, use single-threaded download
        return single_threaded_download(client, url, output_path, args, pb, logger);
    }

    logger.info(&format!(
        "Starting multi-threaded download of {} bytes using {} threads", 
        content_length, args.jobs
    ));

    // Create output directory if needed
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating output directory for {}", output_path))?;
    }

    // Calculate chunk size
    let chunk_size = content_length / args.jobs as u64;
    let mut chunks = Vec::new();
    
    for i in 0..args.jobs {
        let start_byte = i as u64 * chunk_size;
        let end_byte = if i == args.jobs - 1 {
            content_length - 1 // Last chunk gets the remainder
        } else {
            (start_byte + chunk_size) - 1
        };
        
        chunks.push(DownloadChunk {
            start_byte,
            end_byte,
            chunk_id: i,
        });
    }

    // Create temp file path
    let temp_file_path = format!("{}.tmp", output_path);

    // Download chunks in parallel using rayon instead of manual threads    
    let chunk_results: Vec<Result<()>> = chunks.into_par_iter().map(|chunk| {
        download_chunk(client, url, &chunk, &temp_file_path, args, pb.clone(), logger)
    }).collect();

    // Check if all chunks downloaded successfully
    for (i, result) in chunk_results.iter().enumerate() {
        if let Err(e) = result {
            return Err(RuGetError::network(format!(
                "Chunk {} failed to download: {}", i, e
            )));
        }
    }

    // Combine chunks into final file
    combine_chunks(output_path, &temp_file_path, args.jobs, logger)?;

    logger.info(&format!("Multi-threaded download of {} completed successfully", output_path));
    Ok(())
}

/// Single-threaded download fallback
pub fn single_threaded_download(
    client: &Client,
    url: &str,
    output_path: &str,
    args: &Args,
    pb: Option<Arc<ProgressBar>>,
    logger: &Logger,
) -> Result<()> {
    let mut headers = build_headers(&args.headers, logger);
    add_netrc_auth(&mut headers, url).with_context(|| format!("adding netrc auth for {}", url))?;

    // Create output directory if needed
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating output directory for {}", output_path))?;
    }

    // Handle resume logic
    let mut append_mode = false;
    if args.resume && Path::new(output_path).exists() {
        let downloaded = std::fs::metadata(output_path)
            .with_context(|| format!("reading metadata for resume file {}", output_path))?
            .len();

        let head_response = client.head(url).send()
            .with_context(|| format!("fetching remote file length for {}", url))?;
        
        // Try both content_length() method and manual header parsing as fallback
        let auto_len = head_response.content_length();
        let manual_len = head_response.headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        
        // Prefer manual parsing if auto returns 0, otherwise use auto if available
        let remote_len = if auto_len == Some(0) && manual_len.is_some() {
            manual_len.unwrap()
        } else {
            auto_len.or(manual_len).unwrap_or(0)
        };
        
        
        if downloaded >= remote_len {
            logger.info(&format!("File {} already fully downloaded", output_path));
            return Ok(());
        }

        headers.insert(RANGE, format!("bytes={}-", downloaded).parse()
            .with_context(|| format!("creating range header for resume at byte {}", downloaded))?);
        append_mode = true;
        logger.info(&format!("Resuming download from byte {}", downloaded));
    }

    let mut file = if append_mode {
        OpenOptions::new().append(true).open(output_path)
            .with_context(|| format!("opening file in append mode for {}", output_path))?
    } else {
        File::create(output_path)
            .with_context(|| format!("creating new file for {}", output_path))?
    };

    // Create backoff policy from args
    let backoff_policy = BackoffPolicy::with_params(
        Duration::from_millis(args.backoff_base_ms),
        2.0,
        Duration::from_millis(args.backoff_max_ms),
        true,
    );

    let mut attempt = 0;
    loop {
        let response = client.get(url).headers(headers.clone()).send()
            .with_context(|| format!("sending GET request to {}", url));

        match response {
            Ok(mut resp) => {
                let status = resp.status();
                if !status.is_success() && status.as_u16() != 206 {
                    // Treat HTTP status errors as retryable
                    attempt += 1;
                    if attempt > args.max_retries {
                        return Err(RuGetError::network(format!(
                            "{}: failed with HTTP {} after {} retries", url, status, args.max_retries
                        )));
                    }
                    
                    logger.retry_attempt(url, &format!("HTTP {}", status));
                    let delay = backoff_policy.next_delay(attempt - 1);
                    thread::sleep(delay);
                    continue;
                }

                logger.status(url, &status.to_string());
                logger.headers(resp.headers());

                let mut buffer = [0u8; 64 * 1024]; // 64KB buffer
                loop {
                    let n = resp.read(&mut buffer)
                        .with_context(|| format!("reading response data from {}", url))?;
                    if n == 0 {
                        break;
                    }
                    file.write_all(&buffer[..n])
                        .with_context(|| format!("writing data to {}", output_path))?;
                    if let Some(pb) = &pb {
                        pb.inc(n as u64);
                    }
                }

                break;
            }
            Err(e) => {
                attempt += 1;
                if attempt > args.max_retries {
                    let error: RuGetError = e.into();
                    return Err(error.with_context(&format!(
                        "downloading {} after {} retries", url, args.max_retries
                    )));
                }

                // Check if error is transient - for now, retry all errors
                // TODO: When we have actual reqwest::Error, check transience properly
                // if let Some(reqwest_error) = e.source().and_then(|s| s.downcast_ref::<reqwest::Error>()) {
                //     if !is_transient(reqwest_error) {
                //         let error: RuGetError = e.into();
                //         return Err(error.with_context(&format!(
                //             "downloading {} - non-transient error", url
                //         )));
                //     }
                // }

                logger.retry_attempt(url, &e.to_string());
                let delay = backoff_policy.next_delay(attempt - 1);
                thread::sleep(delay);
            }
        }
    }

    Ok(())
}
