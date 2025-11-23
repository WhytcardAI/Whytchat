use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A simple rate limiter using a sliding window algorithm.
///
/// It tracks request timestamps for each unique ID (e.g., session ID or IP address)
/// to determine if a new request is allowed.
pub struct RateLimiter {
    /// Stores timestamps of requests for each client ID.
    requests: HashMap<String, Vec<Instant>>,
    /// The maximum number of requests allowed within the `window`.
    limit: usize,
    /// The duration of the sliding window.
    window: Duration,
}

impl RateLimiter {
    /// Creates a new `RateLimiter`.
    ///
    /// # Arguments
    ///
    /// * `limit` - The number of requests allowed per `window`.
    /// * `window` - The time duration of the sliding window.
    pub fn new(limit: usize, window: Duration) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            limit,
            window,
        }
    }

    /// Checks if a request from a given ID is allowed.
    ///
    /// If the request is allowed, it's recorded and the function returns `true`.
    /// Otherwise, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the source of the request.
    ///
    /// # Returns
    ///
    /// `true` if the request is within the limit, `false` otherwise.
    pub fn check(&mut self, id: &str) -> bool {
        let now = Instant::now();
        let window_start = now - self.window;

        let client_requests = self.requests.entry(id.to_string()).or_default();

        // Remove timestamps older than the window
        client_requests.retain(|&timestamp| timestamp > window_start);

        if client_requests.len() < self.limit {
            client_requests.push(now);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_requests_within_limit() {
        let mut limiter = RateLimiter::new(5, Duration::from_secs(1));
        for _ in 0..5 {
            assert!(limiter.check("client1"));
        }
        assert!(!limiter.check("client1"));
    }

    #[test]
    fn test_rate_limiter_resets_after_window() {
        let mut limiter = RateLimiter::new(2, Duration::from_millis(50));
        assert!(limiter.check("client2"));
        assert!(limiter.check("client2"));
        assert!(!limiter.check("client2"));
        
        thread::sleep(Duration::from_millis(60));
        
        assert!(limiter.check("client2"));
    }
}