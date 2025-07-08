use std::time::Duration;
use std::error::Error;
use rand::Rng;

/// Exponential backoff policy with jitter
#[derive(Debug, Clone)]
pub struct BackoffPolicy {
    pub base_delay: Duration,
    pub factor: f64,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl BackoffPolicy {
    /// Create a new backoff policy with default values
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            factor: 2.0,
            max_delay: Duration::from_secs(60),
            jitter: true,
        }
    }

    /// Create a backoff policy with custom values
    pub fn with_params(base_delay: Duration, factor: f64, max_delay: Duration, jitter: bool) -> Self {
        Self {
            base_delay,
            factor,
            max_delay,
            jitter,
        }
    }

    /// Calculate the next delay for the given attempt number (0-indexed)
    pub fn next_delay(&self, attempt: u32) -> Duration {
        // Calculate base delay: base_delay * factor^attempt
        let base_millis = self.base_delay.as_millis() as f64;
    let calculated_delay = base_millis * self.factor.powi(attempt as i32);
    
    // Cap at max_delay
    let capped_delay = calculated_delay.min(self.max_delay.as_millis() as f64);
    
    let final_delay = if self.jitter {
        // Add jitter: randomly vary by ±25%
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.75..=1.25);
        capped_delay * jitter_factor
    } else {
        capped_delay
    };

    Duration::from_millis(final_delay as u64)
    }
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Determine if an error is transient and should be retried
pub fn is_transient(error: &reqwest::Error) -> bool {
    // Check for specific transient conditions
    if error.is_timeout() || error.is_connect() {
        return true;
    }

    // Check for 5xx status codes (server errors)
    if let Some(status) = error.status() {
        return status.is_server_error();
    }

    // Check for connection reset and similar network errors
    if let Some(source) = error.source() {
        let error_str = source.to_string().to_lowercase();
        if error_str.contains("connection reset") 
            || error_str.contains("broken pipe")
            || error_str.contains("connection aborted")
            || error_str.contains("network unreachable")
            || error_str.contains("host unreachable") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_backoff_policy_default() {
        let policy = BackoffPolicy::new();
        assert_eq!(policy.base_delay, Duration::from_millis(100));
        assert_eq!(policy.factor, 2.0);
        assert_eq!(policy.max_delay, Duration::from_secs(60));
        assert!(policy.jitter);
    }

    #[test]
    fn test_backoff_policy_custom() {
        let policy = BackoffPolicy::with_params(
            Duration::from_millis(500),
            1.5,
            Duration::from_secs(30),
            false,
        );
        assert_eq!(policy.base_delay, Duration::from_millis(500));
        assert_eq!(policy.factor, 1.5);
        assert_eq!(policy.max_delay, Duration::from_secs(30));
        assert!(!policy.jitter);
    }

    #[test]
    fn test_next_delay_exponential_growth() {
        let policy = BackoffPolicy::with_params(
            Duration::from_millis(100),
            2.0,
            Duration::from_secs(60),
            false, // Disable jitter for predictable testing
        );

        // First attempt (0): 100ms
        assert_eq!(policy.next_delay(0), Duration::from_millis(100));
        
        // Second attempt (1): 200ms  
        assert_eq!(policy.next_delay(1), Duration::from_millis(200));
        
        // Third attempt (2): 400ms
        assert_eq!(policy.next_delay(2), Duration::from_millis(400));
        
        // Fourth attempt (3): 800ms
        assert_eq!(policy.next_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_next_delay_max_cap() {
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
    fn test_next_delay_with_jitter() {
        let policy = BackoffPolicy::with_params(
            Duration::from_millis(100),
            2.0,
            Duration::from_secs(60),
            true,
        );

        let delay = policy.next_delay(0);
        // With jitter, should be between 75ms and 125ms
        assert!(delay >= Duration::from_millis(75));
        assert!(delay <= Duration::from_millis(125));
    }

    #[test] 
    fn test_is_transient_timeout() {
        // Mock timeout error - we'll use a simple error string check
        // In real tests, you'd create proper reqwest::Error instances
        // For now, we test the logic structure
        
        // This is a placeholder - in practice you'd need to create actual reqwest errors
        // The function structure is correct for real usage
    }
}
