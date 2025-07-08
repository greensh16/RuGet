use std::process::Command;
use std::fs;

#[test]
fn test_fetch_example_com_to_file() {
    let output_path = "test_output.html";

    let status = Command::new("./target/debug/ruget")
        .args(["https://example.com", "--output", output_path])
        .status()
        .expect("Failed to run ruget");

    assert!(status.success());
    assert!(fs::metadata(output_path).is_ok());

    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains("Example Domain"));

    // Clean up
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_print_to_stdout() {
    let output = Command::new("./target/debug/ruget")
        .arg("https://example.com")
        .output()
        .expect("Failed to run ruget");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Example Domain"));
}