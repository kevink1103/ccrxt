//! # Upbit Rate Limiter Example
//!
//! This example demonstrates how to use the Upbit rate limiter to manage API request limits.
//! The Upbit API has different rate limit groups based on endpoint types:
//!
//! - **Default**: 30 requests per second
//! - **Order**: 8 requests per second
//! - **Market**: 10 requests per second
//! - **Trade**: 10 requests per second
//! - **Ticker**: 10 requests per second
//! - **Candle**: 10 requests per second
//! - **OrderCancelAll**: 1 request per 2 seconds
//!
//! Reference: https://docs.upbit.com/kr/reference/rate-limits

use venues::upbit::{EndpointType, RateLimiter, UpbitRateLimiter};

#[tokio::main]
async fn main() {
    println!("Upbit Rate Limiter Example\n");
    println!("Based on official Upbit API documentation\n");

    // Create a new rate limiter instance
    let limiter = RateLimiter::new();

    // Example 1: Check and record requests for Default endpoints
    println!("Example 1: Default endpoint (30 req/sec)");
    for i in 1..=5 {
        // Check if we can make a request
        match limiter.check_limits_for_endpoint(EndpointType::Default).await {
            Ok(()) => {
                println!("  Request {}: Allowed ✓", i);
                // Record the successful request
                limiter.record_request_for_endpoint(EndpointType::Default).await;
            }
            Err(e) => {
                println!("  Request {}: Rate limit exceeded - {}", i, e);
            }
        }
    }

    // Example 2: Check Order endpoint limits (8 req/sec)
    println!("\nExample 2: Order endpoint (8 req/sec)");
    for i in 1..=10 {
        match limiter.check_limits_for_endpoint(EndpointType::Order).await {
            Ok(()) => {
                println!("  Order request {}: Allowed ✓", i);
                limiter.record_request_for_endpoint(EndpointType::Order).await;
            }
            Err(e) => {
                println!("  Order request {}: Rate limit exceeded - {}", i, e);
                break;
            }
        }
    }

    // Example 3: Check Market endpoint limits (10 req/sec)
    println!("\nExample 3: Market endpoint (10 req/sec)");
    for i in 1..=12 {
        match limiter.check_limits_for_endpoint(EndpointType::Market).await {
            Ok(()) => {
                println!("  Market request {}: Allowed ✓", i);
                limiter.record_request_for_endpoint(EndpointType::Market).await;
            }
            Err(e) => {
                println!("  Market request {}: Rate limit exceeded - {}", i, e);
                break;
            }
        }
    }

    // Example 4: Check Trade endpoint limits (10 req/sec)
    println!("\nExample 4: Trade endpoint (10 req/sec)");
    for i in 1..=5 {
        match limiter.check_limits_for_endpoint(EndpointType::Trade).await {
            Ok(()) => {
                println!("  Trade request {}: Allowed ✓", i);
                limiter.record_request_for_endpoint(EndpointType::Trade).await;
            }
            Err(e) => {
                println!("  Trade request {}: Rate limit exceeded - {}", i, e);
            }
        }
    }

    // Example 5: Check Ticker endpoint limits (10 req/sec)
    println!("\nExample 5: Ticker endpoint (10 req/sec)");
    for i in 1..=5 {
        match limiter.check_limits_for_endpoint(EndpointType::Ticker).await {
            Ok(()) => {
                println!("  Ticker request {}: Allowed ✓", i);
                limiter.record_request_for_endpoint(EndpointType::Ticker).await;
            }
            Err(e) => {
                println!("  Ticker request {}: Rate limit exceeded - {}", i, e);
            }
        }
    }

    // Example 6: Check Candle endpoint limits (10 req/sec)
    println!("\nExample 6: Candle endpoint (10 req/sec)");
    for i in 1..=12 {
        match limiter.check_limits_for_endpoint(EndpointType::Candle).await {
            Ok(()) => {
                println!("  Candle request {}: Allowed ✓", i);
                limiter.record_request_for_endpoint(EndpointType::Candle).await;
            }
            Err(e) => {
                println!("  Candle request {}: Rate limit exceeded - {}", i, e);
                break;
            }
        }
    }

    // Example 7: Check OrderCancelAll endpoint limits (1 req per 2 seconds - very strict!)
    println!("\nExample 7: OrderCancelAll endpoint (1 req per 2 sec)");
    for i in 1..=3 {
        match limiter.check_limits_for_endpoint(EndpointType::OrderCancelAll).await {
            Ok(()) => {
                println!("  OrderCancelAll request {}: Allowed ✓", i);
                limiter.record_request_for_endpoint(EndpointType::OrderCancelAll).await;
            }
            Err(e) => {
                println!("  OrderCancelAll request {}: Rate limit exceeded - {}", i, e);
                break;
            }
        }
    }

    // Example 8: Get usage statistics for all endpoints
    println!("\nExample 8: Usage statistics");
    let stats = limiter.get_endpoint_usage_stats().await;
    for (endpoint_type, (current, limit)) in stats {
        println!("  {:?}: {}/{} requests used", endpoint_type, current, limit);
    }

    // Example 9: Get usage summary
    println!("\nExample 9: Usage summary");
    if let Some(summary) = limiter.get_usage_summary().await {
        println!("  {}", summary);
    }

    println!("\n✓ Example completed successfully!");
    println!("\nNote: OrderCancelAll has the strictest limit at only 1 req per 2 seconds!");
}
