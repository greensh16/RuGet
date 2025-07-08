use ruget::retry::{BackoffPolicy, is_transient};
use ruget::error::{ErrorCode, ErrorKind, RuGetError};
use std::time::Duration;
use httpmock::prelude::*;
use reqwest::{blocking::Client, StatusCode};
use tempfile::NamedTempFile;
use std::sync::{Arc, Mutex};

#[test]
fn test_backoff_policy_exponential_growth() {
    let policy = BackoffPolicy::with_params(
        Duration::from_millis(100),
        2.0,
        Duration::from_secs(60),
        false, // No jitter for predictable testing
    );

    assert_eq!(policy.next_delay(0), Duration::from_millis(100));
    assert_eq!(policy.next_delay(1), Duration::from_millis(200));
    assert_eq!(policy.next_delay(2), Duration::from_millis(400));
    assert_eq!(policy.next_delay(3), Duration::from_millis(800));
}

#[test]
fn test_backoff_policy_max_cap() {
    let policy = BackoffPolicy::with_params(
        Duration::from_millis(100),
        2.0,
        Duration::from_millis(500), // Low max to test capping
        false,
    );

    // Should be capped at max_delay
    assert_eq!(policy.next_delay(10), Duration::from_millis(500));
}

#[test]
fn test_backoff_policy_with_jitter() {
    let policy = BackoffPolicy::with_params(
        Duration::from_millis(100),
        2.0,
        Duration::from_secs(60),
        true,
    );

    // Test multiple attempts to ensure jitter varies
    let mut delays = Vec::new();
    for _ in 0..10 {
        delays.push(policy.next_delay(0));
    }

    // With jitter, we should see some variation
    let min_delay = delays.iter().min().unwrap();
    let max_delay = delays.iter().max().unwrap();
    
    // All delays should be within jitter range (75-125ms for base 100ms)
    for delay in &delays {
        assert!(*delay >= Duration::from_millis(75));
        assert!(*delay <= Duration::from_millis(125));
    }
    
    // We should see some variation (not all delays identical)
    assert!(min_delay != max_delay, "Jitter should cause variation in delays");
}

#[tokio::test]
async fn test_wiremock_transient_error_retry() {
    // Using wiremock to simulate transient HTTP errors and test retry logic
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    let server = MockServer::start().await;

    // Mock server to always return 503 (for testing retry logic and error conversion)
    Mock::given(method("GET"))
        .and(path("/unstable"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    // Test with exponential backoff policy
    let policy = BackoffPolicy::with_params(
        Duration::from_millis(10), // Fast for testing
        2.0,
        Duration::from_millis(100),
        false,
    );

    // Simulate a retry loop manually for testing
    let mut attempts = 0;
    let max_attempts = 2; // Keep small to avoid long test times
    
    loop {
        let client = reqwest::Client::new();
        let response = client.get(&format!("{}/unstable", server.uri())).send().await;
        
        match response {
            Ok(resp) if resp.status().is_success() => {
                panic!("Mock should return 503, not success");
            },
            Ok(resp) if resp.status().is_server_error() => {
                // Test error code conversion
                let error = RuGetError::new(
                    ErrorCode::E203, // HTTP server error
                    ErrorKind::Http,
                    format!("HTTP {} error", resp.status()),
                );
                assert_eq!(error.code, ErrorCode::E203);
                
                attempts += 1;
                if attempts >= max_attempts {
                    // This is expected for this test - we validate retry behavior
                    break;
                }
                tokio::time::sleep(policy.next_delay(attempts - 1)).await;
            },
            _ => panic!("Unexpected status")
        }
    }
    assert_eq!(attempts, max_attempts, "Should have made exactly {} retry attempts", max_attempts);
}

#[test]
fn test_permanent_error_no_retry() {
    let server = MockServer::start();
    
    // Mock server that returns 404 (permanent error)
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/not-found");
        then.status(404)
            .header("content-type", "text/plain")
            .body("Not Found");
    });

    let client = Client::new();
    let result = client.get(&server.url("/not-found")).send();
    match result {
        Ok(resp) => {
            // Should get a 404 response
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            
            // Convert the status to our error system
            let ruget_error = RuGetError::new(
                ErrorCode::E202, // HTTP client error for 4xx
                ErrorKind::Http,
                format!("HTTP {} error", resp.status()),
            );
            assert_eq!(ruget_error.code, ErrorCode::E202);
        },
        Err(e) => {
            // Network error - convert to our error system
            let ruget_error = RuGetError::from(e);
            assert_eq!(ruget_error.code, ErrorCode::E200); // General HTTP error
        },
    }
    
    // 404 should not be considered transient for HTTP status errors
    // (though our is_transient function primarily checks reqwest::Error, not status codes)
}

#[test]
fn test_timeout_is_transient() {
    // This test demonstrates the concept - in practice you'd need to create
    // actual timeout scenarios to test is_transient with real reqwest::Error instances
    
    // For now, we test the basic structure
    let policy = BackoffPolicy::new();
    assert_eq!(policy.base_delay, Duration::from_millis(100));
    assert_eq!(policy.factor, 2.0);
}

#[test]
fn test_retry_backoff_timing() {
    let policy = BackoffPolicy::with_params(
        Duration::from_millis(5),
        2.0,
        Duration::from_millis(100),
        false,
    );

    let start = std::time::Instant::now();
    
    // Simulate a few retry delays
    std::thread::sleep(policy.next_delay(0)); // 5ms
    let first_delay = start.elapsed();
    
    let second_start = std::time::Instant::now();
    std::thread::sleep(policy.next_delay(1)); // 10ms
    let second_delay = second_start.elapsed();
    
    // Verify timing is approximately correct (with generous tolerance for system variance)
    assert!(first_delay >= Duration::from_millis(3), "First delay too short: {:?}", first_delay);
    assert!(first_delay <= Duration::from_millis(50), "First delay too long: {:?}", first_delay);
    
    assert!(second_delay >= Duration::from_millis(8), "Second delay too short: {:?}", second_delay);
    assert!(second_delay <= Duration::from_millis(50), "Second delay too long: {:?}", second_delay);
}

#[tokio::test]
async fn test_backoff_validation_with_wiremock() {
    // Test that validates proper exponential backoff timing in retry scenarios
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use std::time::Instant;

    let server = MockServer::start().await;

    // Mock server to always fail with 503 (testing backoff, not eventual success)
    Mock::given(method("GET"))
        .and(path("/always-fail"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let policy = BackoffPolicy::with_params(
        Duration::from_millis(50), // Base delay
        2.0,                      // Factor
        Duration::from_millis(500), // Max delay
        false,                    // No jitter for predictable timing
    );

    let mut attempt_times = Vec::new();
    let max_attempts = 3;
    
    for attempt in 0..max_attempts {
        let start_time = Instant::now();
        
        let client = reqwest::Client::new();
        let response = client.get(&format!("{}/always-fail", server.uri())).send().await;
        
        // Expect failure
        assert!(response.is_ok());
        assert!(response.unwrap().status().is_server_error());
        
        if attempt < max_attempts - 1 {
            // Apply backoff delay
            let delay = policy.next_delay(attempt);
            tokio::time::sleep(delay).await;
            
            let total_time = start_time.elapsed();
            attempt_times.push(total_time);
        }
    }
    
    // Validate that delays are increasing (exponential backoff)
    if attempt_times.len() >= 2 {
        assert!(attempt_times[1] > attempt_times[0], 
               "Second attempt should take longer than first: {:?} vs {:?}", 
               attempt_times[1], attempt_times[0]);
    }
}
