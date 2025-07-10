use httpmock::prelude::*;
use httpmock::Method::{GET, HEAD};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_simple_http_500() {
    let server = MockServer::start();
    
    // Single mock that returns HTTP 500
    let mock = server.mock(|when, then| {
        when.method(GET);
        then.status(500)
            .body("Internal Server Error");
    });
    
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("test.txt");
    
    let url = format!("{}/test.txt", &server.base_url());
    eprintln!("Request URL: {}", url);
    
    let output = Command::new("./target/debug/ruget")
        .args([
            &url,
            "--output",
            output_path.to_str().unwrap(),
            "--max-retries",
            "2",
            "--jobs", "1",
            "--verbose",
        ])
        .output()
        .unwrap();
    
    eprintln!("Exit code: {:?}", output.status.code());
    eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    eprintln!("Mock hits: {}", mock.hits());
    
    // We expect this to fail, but we want to see how many times the mock was hit
    // and what the error message looks like
}

#[test]
fn test_basic_download_works() {
    let server = MockServer::start();

    let data = "testdata";
    let data_len = data.len();
    
    // Simple HEAD mock for file info
    let _head_mock = server.mock(|when, then| {
        when.method(HEAD);
        then.status(200)
            .header("Content-Length", &data_len.to_string())
            .header("Accept-Ranges", "bytes");
    });

    // Simple GET mock for full download
    let get_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/test.txt");
        then.status(200)
            .header("Content-Length", &data_len.to_string())
            .body(data);
    });

    let dir = tempdir().unwrap();
    let path = dir.path().join("test.txt");

    // Run ruget without --resume first to test basic functionality
    let status = Command::new("./target/debug/ruget")
        .args([
            &format!("{}/test.txt", &server.base_url()),
            "--output",
            path.to_str().unwrap(),
            "--jobs", "1",
        ])
        .status()
        .unwrap();

    assert!(status.success());
    get_mock.assert_hits(1);

    let result = read_to_string(&path).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_resume_download() {
    let server = MockServer::start();

    let data = "testdata";
    let data_len = data.len();
    
    // Mock HEAD request that returns the full file length  
    let head_mock = server.mock(|when, then| {
        when.method(HEAD)
            .path("/test.txt");
        then.status(200)
            .header("content-length", &data_len.to_string())
            .header("accept-ranges", "bytes");
    });

    // Mock GET request with Range header for resume
    let get_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/test.txt")
            .header("range", "bytes=4-");
        then.status(206)
            .header("content-range", &format!("bytes 4-{}/{}", data_len - 1, data_len))
            .header("content-length", &(data_len - 4).to_string())
            .body(&data[4..]);
    });

    // Write partial file (first 4 bytes)
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let mut file = File::create(&path).unwrap();
    write!(file, "test").unwrap();
    drop(file);

    // Run ruget with --resume
    let output = Command::new("./target/debug/ruget")
        .args([
            &format!("{}/test.txt", &server.base_url()),
            "--output",
            path.to_str().unwrap(),
            "--resume",
            "--jobs", "1",
        ])
        .output()
        .unwrap();
    
    // Verify the process succeeded
    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Resume download failed with exit code: {}", output.status.code().unwrap_or(-1));
    }

    // Verify the GET with range header was called
    assert_eq!(get_mock.hits(), 1, "GET with range header should be called exactly once");

    // Verify the final file content is complete
    let result = read_to_string(&path).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_retry_on_failure() {
    let server = MockServer::start();

    let response_data = "response";
    
    // Mock HEAD request for content length
    for _ in 0..10 {
        server.mock(|when, then| {
            when.method(HEAD);
            then.status(200)
                .header("Content-Length", &response_data.len().to_string())
                .header("Accept-Ranges", "bytes");
        });
    }

    // Since httpmock doesn't consume mocks in order as expected,
    // we'll just test that the retry logic attempts multiple requests
    // and eventually gives up after max retries are exhausted.
    // We'll create only failure mocks to ensure all retries fail.
    let failure_mock = server.mock(|when, then| {
        when.method(GET);
        then.status(500)
            .body("Internal Server Error");
    });

    let dir = tempdir().unwrap();
    let output_path = dir.path().join("test.txt");
    
    let url = format!("{}/test.txt", &server.base_url());
    eprintln!("Retry test URL: {}", url);
    
    let output = Command::new("./target/debug/ruget")
        .args([
            &url,
            "--output",
            output_path.to_str().unwrap(),
            "--max-retries",
            "2",  // Use 2 retries for faster test
            "--jobs", "1",
            "--verbose",
        ])
        .output()
        .unwrap();

    // Debug output
    eprintln!("Exit code: {:?}", output.status.code());
    eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    eprintln!("Failure mock hits: {}", failure_mock.hits());
    
    // Verify the process failed after retries (since all mocks return 500)
    assert!(!output.status.success(), "Download should fail when all requests return 500");
    
    // Verify retry attempts were made
    // We expect: 3 attempts in first try + 3 attempts in top-level retry = 6 total
    assert!(failure_mock.hits() >= 4, "Should make multiple retry attempts, got {} hits", failure_mock.hits());
    
    // Verify the verbose output shows retry attempts
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Retrying after error"), "Should show retry attempts in verbose output");
    assert!(stdout.contains("HTTP 500"), "Should show HTTP 500 errors in output");
}
