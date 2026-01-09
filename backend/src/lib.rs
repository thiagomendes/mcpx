//! MCPX Backend Library
//!
//! Shared code for both web server and worker binaries.

pub mod config;
pub mod jobs;
pub mod messages;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;

#[cfg(test)]
pub mod tests;

use std::sync::Arc;

/// Shared application state
pub struct AppState {
    pub config: config::Config,
    pub db: services::db::Database,
    pub rate_limiter: services::rate_limiter::RateLimiterService,
}

impl AppState {
    /// Create new AppState
    pub fn new(config: config::Config, db: services::db::Database) -> Arc<Self> {
        Arc::new(Self {
            config,
            db,
            rate_limiter: services::rate_limiter::RateLimiterService::default(),
        })
    }
}
