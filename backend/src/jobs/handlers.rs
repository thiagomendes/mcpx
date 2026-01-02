//! Job Handlers Module
//!
//! Provides a trait-based abstraction for background job handlers.
//! To add a new job:
//! 1. Implement the `JobHandler` trait
//! 2. Register in `create_job_registry()`
//! 3. Add a row to `job_schedules` table (via migration)
//!
//! ## Example
//!
//! ```ignore
//! struct MyNewJob;
//!
//! #[async_trait]
//! impl JobHandler for MyNewJob {
//!     fn job_type(&self) -> &'static str { "my_new_job" }
//!     
//!     async fn execute(&self, ctx: &JobContext) -> Result<(), String> {
//!         // Do work here
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

// =============================================================================
// CONTEXT
// =============================================================================

/// Context passed to job handlers with dependencies
pub struct JobContext {
    pub pool: PgPool,
    pub encryption_key: String,
}

impl JobContext {
    pub fn new(pool: PgPool, encryption_key: String) -> Self {
        Self {
            pool,
            encryption_key,
        }
    }
}

// =============================================================================
// HANDLER TRAIT
// =============================================================================

/// Trait for job handlers - implement this to create a new job type
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// Unique identifier matching job_schedules.job_type
    fn job_type(&self) -> &'static str;

    /// Execute the job
    async fn execute(&self, ctx: &JobContext) -> Result<(), String>;

    /// Optional: human-readable description
    fn description(&self) -> &'static str {
        "No description"
    }
}

// =============================================================================
// REGISTRY
// =============================================================================

/// Registry of all available job handlers
pub struct JobRegistry {
    handlers: HashMap<String, Arc<dyn JobHandler>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a job handler
    pub fn register<H: JobHandler + 'static>(&mut self, handler: H) {
        let job_type = handler.job_type().to_string();
        self.handlers.insert(job_type, Arc::new(handler));
    }

    /// Get handler for a job type
    pub fn get(&self, job_type: &str) -> Option<Arc<dyn JobHandler>> {
        self.handlers.get(job_type).cloned()
    }

    /// List all registered job types
    pub fn job_types(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// BUILT-IN HANDLERS
// =============================================================================

/// Health check job - pings all MCP servers
pub struct HealthCheckHandler;

#[async_trait]
impl JobHandler for HealthCheckHandler {
    fn job_type(&self) -> &'static str {
        "health_check"
    }

    fn description(&self) -> &'static str {
        "Checks connectivity to all registered MCP servers"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<(), String> {
        let db = crate::services::db::Database {
            pool: ctx.pool.clone(),
        };
        crate::services::health_check::run_health_check_cycle(&db, &ctx.encryption_key)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Alert evaluation job - checks alert rules and triggers notifications
pub struct AlertEvaluatorHandler;

#[async_trait]
impl JobHandler for AlertEvaluatorHandler {
    fn job_type(&self) -> &'static str {
        "alert_eval"
    }

    fn description(&self) -> &'static str {
        "Evaluates alert rules and triggers notifications"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<(), String> {
        crate::jobs::alert_evaluator::run_evaluation_cycle(&ctx.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

// =============================================================================
// FACTORY
// =============================================================================

/// Create a registry with all built-in job handlers
pub fn create_job_registry() -> JobRegistry {
    let mut registry = JobRegistry::new();

    // Register all built-in handlers
    registry.register(HealthCheckHandler);
    registry.register(AlertEvaluatorHandler);

    // To add a new job:
    // 1. Create a struct implementing JobHandler
    // 2. Add registry.register(YourNewHandler);
    // 3. Add migration to insert into job_schedules

    registry
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler;

    #[async_trait]
    impl JobHandler for TestHandler {
        fn job_type(&self) -> &'static str {
            "test_job"
        }
        async fn execute(&self, _ctx: &JobContext) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = JobRegistry::new();
        registry.register(TestHandler);

        assert!(registry.get("test_job").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_job_types() {
        let mut registry = JobRegistry::new();
        registry.register(TestHandler);

        let types = registry.job_types();
        assert!(types.contains(&"test_job"));
    }

    #[test]
    fn test_create_job_registry() {
        let registry = create_job_registry();

        assert!(registry.get("health_check").is_some());
        assert!(registry.get("alert_eval").is_some());
    }
}
