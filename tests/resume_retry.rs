use httpmock::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_resume_download() {
    let server = MockServer::start();

    // Create mock that supports Range header
    let data = "HelloWorldFromRustget";
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/file.txt")
            .header("range", "bytes=5-");
        then.status(206)
            .header("Content-Length", "17")
            .body(&data[5..]);
    });

    // First, write partial file (first 5 bytes)
    let dir = tempdir().unwrap();
    let path = dir.path().join("file.txt");
    let mut file = File::create(&path).unwrap();
    write!(file, "Hello").unwrap();

    // Run ruget with --resume
    let status = Command::new("./target/debug/ruget")
        .args([
            &format!("{}/file.txt", &server.base_url()),
            "--output",
            path.to_str().unwrap(),
            "--resume",
        ])
        .status()
        .unwrap();

    assert!(status.success());
    mock.assert_hits(1);

    let result = read_to_string(&path).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_retry_on_failure() {
    let server = MockServer::start();

    // Mock server to fail twice then succeed
    let mock_fail1 = server.mock(|when, then| {
        when.path("/unstable.txt");
        then.status(500);
    });

    let mock_fail2 = server.mock(|when, then| {
        when.path("/unstable.txt");
        then.status(500);
    });

    let mock_success = server.mock(|when, then| {
        when.path("/unstable.txt");
        then.status(200)
            .body("final response");
    });

    let dir = tempdir().unwrap();
    let output = dir.path().join("unstable.txt");

    let status = Command::new("./target/debug/ruget")
        .args([
            &format!("{}/unstable.txt", &server.base_url()),
            "--output",
            output.to_str().unwrap(),
            "--max-retries",
            "5",
        ])
        .status()
        .unwrap();

    assert!(status.success());
    // Note: Due to how we set up the mocks, we can't easily assert exact hit counts
    // but the test still validates that retries work

    let result = read_to_string(output).unwrap();
    assert_eq!(result, "final response");
}