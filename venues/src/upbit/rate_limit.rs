use std::time::{Duration, Instant};

use async_trait::async_trait;
use parking_lot::RwLock;
use thiserror::Error;

use super::errors::Errors;
use super::rate_limiter_trait::UpbitRateLimiter;

/// Types of endpoints for rate limiting based on Upbit's rate limit groups
/// Reference: https://docs.upbit.com/kr/reference/rate-limits
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EndpointType {
    /// Default endpoints - 30 requests per second
    Default,
    /// Order endpoints - 8 requests per second
    Order,
    /// Market endpoints - 10 requests per second
    Market,
    /// Trade endpoints (orderbook, trades) - 10 requests per second
    Trade,
    /// Ticker endpoints (current price) - 10 requests per second
    Ticker,
    /// Candle endpoints - 10 requests per second
    Candle,
    /// Order cancel all endpoint - 1 request per 2 seconds
    OrderCancelAll,
}

/// Rate limit configuration for different endpoint types
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window: Duration,
}

impl RateLimit {
    /// Create a new rate limit configuration
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

/// Rate limiting errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RateLimitError {
    #[error("Rate limit exceeded for endpoint type: {endpoint_type:?}")]
    Exceeded { endpoint_type: EndpointType },
}

/// Rate limiter for Upbit API endpoints
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Request timestamps for different endpoint types
    request_history: std::sync::Arc<RwLock<std::collections::HashMap<EndpointType, Vec<Instant>>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            request_history: std::sync::Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get rate limit for endpoint type
    fn get_rate_limit(endpoint_type: &EndpointType) -> RateLimit {
        match endpoint_type {
            EndpointType::Default => RateLimit::new(30, Duration::from_secs(1)),
            EndpointType::Order => RateLimit::new(8, Duration::from_secs(1)),
            EndpointType::Market => RateLimit::new(10, Duration::from_secs(1)),
            EndpointType::Trade => RateLimit::new(10, Duration::from_secs(1)),
            EndpointType::Ticker => RateLimit::new(10, Duration::from_secs(1)),
            EndpointType::Candle => RateLimit::new(10, Duration::from_secs(1)),
            EndpointType::OrderCancelAll => RateLimit::new(1, Duration::from_secs(2)),
        }
    }

    /// Check if a request can be made
    pub async fn check_limits(&self, endpoint_type: EndpointType) -> Result<(), RateLimitError> {
        let rate_limit = Self::get_rate_limit(&endpoint_type);
        let mut history = self.request_history.write();
        let now = Instant::now();

        // Get or create history for this endpoint type
        let timestamps = history.entry(endpoint_type.clone()).or_default();

        // Remove old timestamps outside the window
        timestamps.retain(|&timestamp| now.duration_since(timestamp) < rate_limit.window);

        // Check if we're at the limit
        if timestamps.len() >= rate_limit.max_requests as usize {
            return Err(RateLimitError::Exceeded { endpoint_type });
        }

        Ok(())
    }

    /// Record a request
    pub async fn increment_request(&self, endpoint_type: EndpointType) {
        let mut history = self.request_history.write();
        let timestamps = history.entry(endpoint_type).or_default();
        timestamps.push(Instant::now());
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the Upbit-specific trait
#[async_trait]
impl UpbitRateLimiter for RateLimiter {
    async fn check_limits_for_endpoint(&self, endpoint_type: EndpointType) -> Result<(), Errors> {
        self.check_limits(endpoint_type).await.map_err(|e| match e {
            RateLimitError::Exceeded { endpoint_type } => {
                Errors::Error(format!("Rate limit exceeded for {:?}", endpoint_type))
            }
        })
    }

    async fn record_request_for_endpoint(&self, endpoint_type: EndpointType) {
        self.increment_request(endpoint_type).await;
    }

    async fn get_endpoint_usage_stats(
        &self,
    ) -> std::collections::HashMap<EndpointType, (u32, u32)> {
        let history = self.request_history.read();
        let mut stats = std::collections::HashMap::new();
        for (endpoint_type, timestamps) in history.iter() {
            let rate_limit = Self::get_rate_limit(endpoint_type);
            stats.insert(
                endpoint_type.clone(),
                (timestamps.len() as u32, rate_limit.max_requests),
            );
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new();

        // First request should succeed
        assert!(limiter.check_limits(EndpointType::Default).await.is_ok());
        limiter.increment_request(EndpointType::Default).await;
    }

    #[tokio::test]
    async fn test_order_rate_limit() {
        let limiter = RateLimiter::new();

        // Test order endpoint rate limiting
        assert!(limiter.check_limits(EndpointType::Order).await.is_ok());
        limiter.increment_request(EndpointType::Order).await;
    }

    #[tokio::test]
    async fn test_candle_rate_limit() {
        let limiter = RateLimiter::new();

        // Test candle endpoint rate limiting (10 per second)
        for _ in 0..10 {
            assert!(limiter.check_limits(EndpointType::Candle).await.is_ok());
            limiter.increment_request(EndpointType::Candle).await;
        }

        // 11th request should fail
        assert!(limiter.check_limits(EndpointType::Candle).await.is_err());
    }

    #[tokio::test]
    async fn test_trade_rate_limit() {
        let limiter = RateLimiter::new();

        // Test trade endpoint rate limiting
        assert!(limiter.check_limits(EndpointType::Trade).await.is_ok());
        limiter.increment_request(EndpointType::Trade).await;
    }

    #[tokio::test]
    async fn test_ticker_rate_limit() {
        let limiter = RateLimiter::new();

        // Test ticker endpoint rate limiting
        assert!(limiter.check_limits(EndpointType::Ticker).await.is_ok());
        limiter.increment_request(EndpointType::Ticker).await;
    }

    #[tokio::test]
    async fn test_order_cancel_all_rate_limit() {
        let limiter = RateLimiter::new();

        // Test order cancel all endpoint rate limiting (1 per 2 seconds)
        assert!(limiter.check_limits(EndpointType::OrderCancelAll).await.is_ok());
        limiter.increment_request(EndpointType::OrderCancelAll).await;

        // Second request should fail (limit is 1 per 2 seconds)
        assert!(limiter.check_limits(EndpointType::OrderCancelAll).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_config() {
        let config = RateLimit::new(100, Duration::from_secs(60));
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_endpoint_types() {
        let default = EndpointType::Default;
        let order = EndpointType::Order;
        let candle = EndpointType::Candle;
        let trade = EndpointType::Trade;
        let ticker = EndpointType::Ticker;

        assert_ne!(default, order);
        assert_ne!(default, candle);
        assert_ne!(trade, ticker);
        assert_eq!(default.clone(), EndpointType::Default);
        assert_eq!(candle.clone(), EndpointType::Candle);
    }

    #[tokio::test]
    async fn test_rate_limit_exceeded() {
        let limiter = RateLimiter::new();

        // Fill up the limit for Order endpoint (8 requests per second)
        for _ in 0..8 {
            assert!((limiter.check_limits(EndpointType::Order).await).is_ok());
            limiter.increment_request(EndpointType::Order).await;
        }

        // Next request should fail
        assert!(limiter.check_limits(EndpointType::Order).await.is_err());
    }

    #[tokio::test]
    async fn test_different_endpoints_independent() {
        let limiter = RateLimiter::new();

        // Use up Order endpoint
        for _ in 0..8 {
            limiter.increment_request(EndpointType::Order).await;
        }

        // Default endpoint should still work
        assert!(limiter.check_limits(EndpointType::Default).await.is_ok());

        // Trade endpoint should still work
        assert!(limiter.check_limits(EndpointType::Trade).await.is_ok());
    }

    #[tokio::test]
    async fn test_market_trade_ticker_limits() {
        let limiter = RateLimiter::new();

        // Fill up Market endpoint (10 per second)
        for _ in 0..10 {
            assert!(limiter.check_limits(EndpointType::Market).await.is_ok());
            limiter.increment_request(EndpointType::Market).await;
        }
        assert!(limiter.check_limits(EndpointType::Market).await.is_err());

        // Fill up Trade endpoint (10 per second)
        for _ in 0..10 {
            assert!(limiter.check_limits(EndpointType::Trade).await.is_ok());
            limiter.increment_request(EndpointType::Trade).await;
        }
        assert!(limiter.check_limits(EndpointType::Trade).await.is_err());

        // Fill up Ticker endpoint (10 per second)
        for _ in 0..10 {
            assert!(limiter.check_limits(EndpointType::Ticker).await.is_ok());
            limiter.increment_request(EndpointType::Ticker).await;
        }
        assert!(limiter.check_limits(EndpointType::Ticker).await.is_err());
    }
}
