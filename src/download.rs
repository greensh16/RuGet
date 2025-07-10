use crate::cli::Args;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use reqwest::{
    blocking::Client,
    cookie::Jar,
};
use crate::output::Logger;
use crate::error::{Result, RuGetError, WithContext};
use crate::http::{build_headers, extract_filename_from_disposition, get_fallback_filename};
use crate::multithreaded_download::{multithreaded_download_url, single_threaded_download};
use crate::cookie::{load_cookies_from_file, save_cookies_to_file};

#[cfg(feature = "context")]
use crate::error::AnyhowContextExt;



pub fn download(args: Args, logger: &Logger) -> Result<()> {
    let cookie_jar = Arc::new(Jar::default());

    // Load cookies from file if specified
    if let Some(cookie_file) = &args.load_cookies {
        load_cookies_from_file(&cookie_jar, cookie_file, logger)?;
    }

    let client = Client::builder()
        .cookie_provider(cookie_jar.clone())
        .default_headers(build_headers(&args.headers, &logger))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .with_context(|| "building HTTP client".to_string())?;

    if args.urls.len() > 1 && args.output.is_some() {
        return Err(RuGetError::parse("Cannot use --output with multiple URLs".into()));
    }

    let total_size: u64 = args
        .urls
        .iter()
        .filter_map(|url| client.head(url).send().ok()?.content_length())
        .sum();

    let global_pb = if !args.quiet {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("=> "),
        );
        Some(Arc::new(pb))
    } else {
        None
    };

    let urls = args.urls.clone();
    let args = Arc::new(args);
    let client = Arc::new(client);
    let failures = Arc::new(Mutex::new(Vec::new()));

    if args.jobs > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.jobs)
            .build_global()
            .ok();
    }

    let results: Vec<_> = urls
        .into_par_iter()
        .map(|url| {
            let args = Arc::clone(&args);
            let client = Arc::clone(&client);
            let failures = Arc::clone(&failures);
            let global_pb = global_pb.clone();

            let head_resp = client.head(&url).send().ok();
            let suggested_name = head_resp
                .as_ref()
                .and_then(|resp| extract_filename_from_disposition(resp.headers().get("content-disposition")));

            let final_name = suggested_name.unwrap_or_else(|| get_fallback_filename(&url));

            let output_path = if let Some(ref path) = args.output {
                path.clone()
            } else if let Some(dir) = &args.output_dir {
                let mut path = PathBuf::from(dir);
                path.push(final_name);
                path.to_string_lossy().into_owned()
            } else {
                final_name
            };

            logger.download_start(&url, &output_path);

            let url_clone = url.clone();
            let result = if args.jobs > 1 {
                multithreaded_download_url(&client, &url, &output_path, &args, global_pb.clone(), &logger)
            } else {
                single_threaded_download(&client, &url, &output_path, &args, global_pb.clone(), &logger)
            };
            
            match result {
                Ok(_) => Some(url_clone),
                Err(err) => {
                    failures.lock().unwrap().push((url_clone, output_path.clone()));
                    logger.error_from_ruget_error(&err);
                    None
                }
            }
        })
        .collect();

    if let Some(pb) = &global_pb {
        pb.finish_and_clear();
    }

    let num_success = results.iter().filter(|r| r.is_some()).count();
    let total = results.len();
    logger.summary(num_success, total);

    let failures = Arc::try_unwrap(failures).unwrap().into_inner().unwrap();
    let mut final_failures = vec![];

    for (url, output_path) in failures {
        logger.info(&format!("Retrying: {}", url));

        let retry_result = if args.jobs > 1 {
            multithreaded_download_url(&client, &url, &output_path, &args, global_pb.clone(), &logger)
        } else {
            single_threaded_download(&client, &url, &output_path, &args, global_pb.clone(), &logger)
        };
        
        match retry_result {
            Ok(_) => {
                logger.info(&format!("Retry succeeded: {}", url));
            }
            Err(e) => {
                let mut context = HashMap::new();
                context.insert("url".to_string(), url.clone());
                context.insert("attempt".to_string(), "retry".to_string());
                
                // Since e is already RuGetError type, use error_from_ruget_error directly
                logger.error_from_ruget_error(&e);
                final_failures.push((url, e.to_string()));
            }
        }
    }

    if !final_failures.is_empty() {
        let mut log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.log)
            .with_context(|| format!("opening log file {}", args.log))?;

        for (url, err) in &final_failures {
            writeln!(log, "{}\t{}", url, err)
                .with_context(|| format!("writing failure entry to log file {}", args.log))?;
        }

        logger.warn(&format!(
            "{} downloads permanently failed. See {} for details.",
            final_failures.len(),
            args.log
        ));
        
        // If all downloads failed, return an error
        if final_failures.len() == total {
            // Save cookies before returning error
            if let Some(cookie_file) = &args.save_cookies {
                save_cookies_to_file(&cookie_jar, cookie_file, args.keep_session_cookies, logger)?;
            }
            return Err(RuGetError::network("All downloads failed after retries".into()));
        }
    }

    // Save cookies to file if specified
    if let Some(cookie_file) = &args.save_cookies {
        save_cookies_to_file(&cookie_jar, cookie_file, args.keep_session_cookies, logger)?;
    }

    Ok(())
}
