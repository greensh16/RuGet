use crate::cli::Args;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use rayon::prelude::*;
use netrc::Netrc;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write, BufReader},
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
    sync::{Arc, Mutex},
};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue, RANGE, AUTHORIZATION},
};

fn build_headers(header_args: &[String]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for h in header_args {
        if let Some((k, v)) = h.split_once(':') {
            let name = k.trim().parse::<HeaderName>().unwrap();
            let value = v.trim().parse::<HeaderValue>().unwrap();
            headers.insert(name, value);
        }
    }
    headers
}

fn maybe_add_netrc_auth(headers: &mut HeaderMap, url: &str) {
    if let Ok(parsed_url) = reqwest::Url::parse(url) {
        if let Some(host) = parsed_url.host_str() {
            let home = std::env::var("HOME").unwrap_or_default();
            let netrc_path = format!("{}/.netrc", home);
            if let Ok(file) = File::open(netrc_path) {
                if let Ok(netrc) = Netrc::parse(BufReader::new(file)) {
                    if let Some((_, machine)) = netrc.hosts.iter().find(|(h, _)| h == host) {
                        if !machine.login.is_empty() {
                            if let Some(password) = &machine.password {
                                if !password.is_empty() {
                                    let encoded = BASE64_STANDARD.encode(format!(
                                        "{}:{}",
                                        machine.login, password
                                    ));
                                    let auth = format!("Basic {}", encoded);
                                    headers.insert(AUTHORIZATION, auth.parse().unwrap());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_filename_from_disposition(header: Option<&HeaderValue>) -> Option<String> {
    if let Some(value) = header {
        if let Ok(value_str) = value.to_str() {
            let re = Regex::new(r#"filename="?([^"]+)"?"#).unwrap();
            if let Some(cap) = re.captures(value_str) {
                return Some(cap[1].to_string());
            }
        }
    }
    None
}

fn download_url(
    client: &Client,
    url: &str,
    output_path: &str,
    args: &Args,
    global_pb: Option<Arc<ProgressBar>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = build_headers(&args.headers);
    maybe_add_netrc_auth(&mut headers, url);

    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Determine resume position
    let mut append_mode = false;

    let remote_len = client.head(url).send()?.content_length().unwrap_or(0);

    if args.resume && Path::new(output_path).exists() {
        let downloaded = std::fs::metadata(output_path)?.len();

        if downloaded >= remote_len {
            if !args.quiet {
                println!("Already downloaded: {}", output_path);
            }
            return Ok(());
        }

        headers.insert(RANGE, format!("bytes={}-", downloaded).parse()?);
        append_mode = true;

        if !args.quiet {
            println!("Resuming {} from byte {}", output_path, downloaded);
        }
    }

    let mut file = if append_mode {
        OpenOptions::new().append(true).open(output_path)?
    } else {
        File::create(output_path)?
    };

    let mut retries = args.retries;
    loop {
        let response = client.get(url).headers(headers.clone()).send();

        match response {
            Ok(mut resp) => {
                let status = resp.status();
                if !status.is_success() && status.as_u16() != 206 {
                    return Err(format!("{}: failed with HTTP {}", url, status).into());
                }

                if !args.quiet {
                    println!("{}: {}", url, status);
                    if args.verbose {
                        for (k, v) in resp.headers() {
                            println!("  {}: {}", k, v.to_str().unwrap_or("[binary]"));
                        }
                    }
                }

                let pb = global_pb.unwrap_or_else(|| Arc::new(ProgressBar::hidden()));
                let mut buffer = [0u8; 8192];
                loop {
                    let n = resp.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    file.write_all(&buffer[..n])?;
                    pb.inc(n as u64);
                }

                break;
            }
            Err(e) => {
                retries -= 1;
                if retries == 0 {
                    return Err(e.into());
                }
                if !args.quiet {
                    eprintln!("Retrying after error: {}", e);
                }
                sleep(Duration::from_secs(2));
            }
        }
    }

    Ok(())
}

pub fn download(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let headers = build_headers(&args.headers);

    let client = Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    if args.urls.len() > 1 && args.output.is_some() {
        return Err("Cannot use --output with multiple URLs".into());
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

            let fallback_name = url
                .split('/')
                .last()
                .filter(|s| !s.is_empty())
                .unwrap_or("download.bin")
                .to_string();

            let final_name = suggested_name.unwrap_or(fallback_name);

            let output_path = if let Some(ref path) = args.output {
                path.clone()
            } else if let Some(dir) = &args.output_dir {
                let mut path = PathBuf::from(dir);
                path.push(final_name);
                path.to_string_lossy().into_owned()
            } else {
                final_name
            };

            if !args.quiet {
                println!("Saving {} to {}", url, output_path);
            }

            match download_url(&client, &url, &output_path, &args, global_pb) {
                Ok(_) => Some(url),
                Err(_) => {
                    failures.lock().unwrap().push((url.clone(), output_path));
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
    if !args.quiet {
        println!("{}/{} downloads succeeded (initial pass)", num_success, total);
    }

    let failures = Arc::try_unwrap(failures).unwrap().into_inner().unwrap();
    let mut final_failures = vec![];

    for (url, output_path) in failures {
        if !args.quiet {
            println!("Retrying: {}", url);
        }

        match download_url(&client, &url, &output_path, &args, global_pb.clone()) {
            Ok(_) => {
                if !args.quiet {
                    println!("Retry succeeded: {}", url);
                }
            }
            Err(e) => {
                if !args.quiet {
                    println!("Retry failed: {} ({})", url, e);
                }
                final_failures.push((url, e.to_string()));
            }
        }
    }

    if !final_failures.is_empty() {
        let mut log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.log)?;

        for (url, err) in &final_failures {
            writeln!(log, "{}\t{}", url, err)?;
        }

        println!(
            "{} downloads permanently failed. See {} for details.",
            final_failures.len(),
            args.log
        );
    }

    Ok(())
}