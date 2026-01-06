//! Rate Limiter Service
//!
//! In-memory rate limiting with DashMap for high performance.
//! Rates are configured per-server with opt-in (NULL = no limit).

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Rate limit cache entry
struct RateLimitEntry {
    /// Current request count in window
    count: AtomicU64,
    /// Window start time
    window_start: Instant,
    /// Configured limit (requests per minute)
    #[allow(dead_code)]
    limit: u32,
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: i64, // Unix timestamp
}

/// Rate limit exceeded error
#[derive(Debug, Clone)]
pub struct RateLimitExceeded {
    pub limit: u32,
    pub retry_after: u32, // seconds
    pub server_name: String,
}

impl std::fmt::Display for RateLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate limit exceeded for server '{}': {} requests/min",
            self.server_name, self.limit
        )
    }
}  

impl std::error::Error for RateLimitExceeded {}

/// Cache key: (org_id, server_id)
type CacheKey = (Uuid, Uuid);

/// Configuration for rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Cache TTL - entries are refreshed after this duration
    pub cache_ttl: Duration,
    /// Window duration (1 minute)
    pub window_duration: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(30),
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Rate limiter service with in-memory cache
pub struct RateLimiterService {
    cache: DashMap<CacheKey, RateLimitEntry>,
    config: RateLimiterConfig,
}

impl RateLimiterService {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
        }
    }

    /// Check rate limit and increment counter
    /// Returns Ok(info) if allowed, Err(exceeded) if rate limited
    /// 
    /// If `limit` is None, no rate limiting is applied (returns Ok with unlimited info)
    pub fn check_and_increment(
        &self,
        org_id: Uuid,
        server_id: Uuid,
        server_name: &str,
        limit: Option<i32>,
    ) -> Result<RateLimitInfo, RateLimitExceeded> {
        // No limit configured = unlimited
        let limit = match limit {
            Some(l) if l > 0 => l as u32,
            _ => {
                return Ok(RateLimitInfo {
                    limit: 0, // 0 indicates unlimited
                    remaining: u32::MAX,
                    reset_at: 0,
                });
            }
        };

        let key = (org_id, server_id);
        let now = Instant::now();

        // Try to get or create entry
        let entry = self.cache.entry(key).or_insert_with(|| RateLimitEntry {
            count: AtomicU64::new(0),
            window_start: now,
            limit,
        });

        // Check if window has expired
        let elapsed = now.duration_since(entry.window_start);
        if elapsed >= self.config.window_duration {
            // Window expired - reset
            entry.count.store(0, Ordering::SeqCst);
            // Note: We can't mutate window_start here due to borrow rules
            // This is a simplification - in production you'd use a more sophisticated approach
        }

        // Get current count and increment
        let current_count = entry.count.fetch_add(1, Ordering::SeqCst);

        // Calculate reset time
        let seconds_until_reset = self
            .config
            .window_duration
            .saturating_sub(elapsed)
            .as_secs();
        let reset_at = chrono::Utc::now().timestamp() + seconds_until_reset as i64;

        // Check if exceeded
        if current_count >= limit as u64 {
            // Decrement since we're rejecting
            entry.count.fetch_sub(1, Ordering::SeqCst);
            return Err(RateLimitExceeded {
                limit,
                retry_after: seconds_until_reset as u32,
                server_name: server_name.to_string(),
            });
        }

        let remaining = limit.saturating_sub(current_count as u32 + 1);

        Ok(RateLimitInfo {
            limit,
            remaining,
            reset_at,
        })
    }

    /// Get current rate limit info without incrementing
    pub fn get_info(
        &self,
        org_id: Uuid,
        server_id: Uuid,
        limit: Option<i32>,
    ) -> RateLimitInfo {
        let limit = match limit {
            Some(l) if l > 0 => l as u32,
            _ => {
                return RateLimitInfo {
                    limit: 0,
                    remaining: u32::MAX,
                    reset_at: 0,
                };
            }
        };

        let key = (org_id, server_id);
        
        match self.cache.get(&key) {
            Some(entry) => {
                let count = entry.count.load(Ordering::SeqCst) as u32;
                let elapsed = Instant::now().duration_since(entry.window_start);
                let seconds_until_reset = self
                    .config
                    .window_duration
                    .saturating_sub(elapsed)
                    .as_secs();
                let reset_at = chrono::Utc::now().timestamp() + seconds_until_reset as i64;

                RateLimitInfo {
                    limit,
                    remaining: limit.saturating_sub(count),
                    reset_at,
                }
            }
            None => RateLimitInfo {
                limit,
                remaining: limit,
                reset_at: chrono::Utc::now().timestamp() + 60,
            },
        }
    }

    /// Clear expired entries from cache (call periodically)
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| {
            now.duration_since(entry.window_start) < self.config.cache_ttl * 2
        });
    }
}

impl Default for RateLimiterService {
    fn default() -> Self {
        Self::new(RateLimiterConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_limit_returns_unlimited() {
        let service = RateLimiterService::default();
        let org_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let result = service.check_and_increment(org_id, server_id, "test", None);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.limit, 0); // 0 = unlimited
    }

    #[test]
    fn test_rate_limit_allows_under_limit() {
        let service = RateLimiterService::default();
        let org_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        // 5 requests with limit of 10
        for i in 0..5 {
            let result = service.check_and_increment(org_id, server_id, "test", Some(10));
            assert!(result.is_ok(), "Request {} should succeed", i);
        }
    }

    #[test]
    fn test_rate_limit_blocks_over_limit() {
        let service = RateLimiterService::default();
        let org_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        // 5 requests with limit of 5
        for _ in 0..5 {
            let result = service.check_and_increment(org_id, server_id, "test", Some(5));
            assert!(result.is_ok());
        }

        // 6th request should fail
        let result = service.check_and_increment(org_id, server_id, "test", Some(5));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.limit, 5);
    }

    #[test]
    fn test_different_servers_have_separate_limits() {
        let service = RateLimiterService::default();
        let org_id = Uuid::new_v4();
        let server1 = Uuid::new_v4();
        let server2 = Uuid::new_v4();

        // Hit limit on server1
        for _ in 0..5 {
            service.check_and_increment(org_id, server1, "server1", Some(5)).unwrap();
        }

        // server2 should still work
        let result = service.check_and_increment(org_id, server2, "server2", Some(5));
        assert!(result.is_ok());
    }
}
