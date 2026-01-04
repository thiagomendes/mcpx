//! Job Queue Service
//!
//! Provides distributed job scheduling with PostgreSQL SKIP LOCKED.
//!
//! ## Architecture
//!
//! This module uses a trait-based design to allow future replacement
//! of the job backend (e.g., Redis, SQS) without changing worker code.
//!
//! ## Usage
//!
//! ```ignore
//! let queue = PostgresJobQueue::new(pool);
//! if let Some(job) = queue.claim_next("worker-1").await? {
//!     // Execute job...
//!     queue.complete(job.id, job.interval_seconds).await?;
//! }
//! ```

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// =============================================================================
// TYPES
// =============================================================================

/// A claimed job ready for execution
#[derive(Debug, Clone)]
pub struct ClaimedJob {
    pub id: Uuid,
    pub job_type: String,
    pub interval_seconds: i64,
}

/// Result type for job queue operations
pub type JobQueueResult<T> = Result<T, JobQueueError>;

/// Job queue errors
#[derive(Debug)]
pub enum JobQueueError {
    Database(String),
    InvalidJobType(String),
}

impl std::fmt::Display for JobQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobQueueError::Database(msg) => write!(f, "Database error: {}", msg),
            JobQueueError::InvalidJobType(t) => write!(f, "Invalid job type: {}", t),
        }
    }
}

impl std::error::Error for JobQueueError {}

impl From<sqlx::Error> for JobQueueError {
    fn from(e: sqlx::Error) -> Self {
        JobQueueError::Database(e.to_string())
    }
}

// =============================================================================
// TRAIT (for future swappability)
// =============================================================================

/// Job queue abstraction - implement this trait to use a different backend
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Claim the next available job that's due to run.
    /// Returns None if no jobs are available.
    async fn claim_next(&self, worker_id: &str) -> JobQueueResult<Option<ClaimedJob>>;

    /// Mark a job as complete and schedule the next run.
    async fn complete(&self, job_id: Uuid, interval_seconds: i64) -> JobQueueResult<()>;

    /// Release a job lock without completing (on error).
    async fn release(&self, job_id: Uuid) -> JobQueueResult<()>;

    /// Check if a specific job type is enabled.
    async fn is_enabled(&self, job_type: &str) -> JobQueueResult<bool>;
}

// =============================================================================
// POSTGRESQL IMPLEMENTATION
// =============================================================================

/// PostgreSQL-based job queue using SKIP LOCKED for distributed locking
pub struct PostgresJobQueue {
    pool: PgPool,
    lock_timeout_seconds: i64,
}

impl PostgresJobQueue {
    /// Create a new PostgreSQL job queue
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            lock_timeout_seconds: 300, // 5 minutes
        }
    }

    /// Create with custom lock timeout
    pub fn with_timeout(pool: PgPool, lock_timeout_seconds: i64) -> Self {
        Self {
            pool,
            lock_timeout_seconds,
        }
    }
}

#[async_trait]
impl JobQueue for PostgresJobQueue {
    async fn claim_next(&self, worker_id: &str) -> JobQueueResult<Option<ClaimedJob>> {
        // Use CTE with FOR UPDATE SKIP LOCKED for atomic claim
        let job = sqlx::query_as::<_, (Uuid, String, i32)>(
            r#"
            WITH claimable AS (
                SELECT id, job_type, interval_seconds
                FROM job_schedules
                WHERE enabled = true
                  AND next_run_at <= NOW()
                  AND (locked_by IS NULL OR locked_at < NOW() - INTERVAL '1 second' * $2)
                ORDER BY next_run_at
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            UPDATE job_schedules js
            SET 
                locked_by = $1,
                locked_at = NOW(),
                updated_at = NOW()
            FROM claimable c
            WHERE js.id = c.id
            RETURNING js.id, js.job_type, js.interval_seconds
        "#,
        )
        .bind(worker_id)
        .bind(self.lock_timeout_seconds)
        .fetch_optional(&self.pool)
        .await?;

        Ok(job.map(|(id, job_type, interval)| ClaimedJob {
            id,
            job_type,
            interval_seconds: interval as i64,
        }))
    }

    async fn complete(&self, job_id: Uuid, interval_seconds: i64) -> JobQueueResult<()> {
        sqlx::query(
            r#"
            UPDATE job_schedules
            SET 
                locked_by = NULL,
                locked_at = NULL,
                last_run_at = NOW(),
                next_run_at = NOW() + INTERVAL '1 second' * $2,
                updated_at = NOW()
            WHERE id = $1
        "#,
        )
        .bind(job_id)
        .bind(interval_seconds)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn release(&self, job_id: Uuid) -> JobQueueResult<()> {
        sqlx::query(
            r#"
            UPDATE job_schedules
            SET 
                locked_by = NULL,
                locked_at = NULL,
                updated_at = NOW()
            WHERE id = $1
        "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn is_enabled(&self, job_type: &str) -> JobQueueResult<bool> {
        let enabled: Option<bool> =
            sqlx::query_scalar("SELECT enabled FROM job_schedules WHERE job_type = $1")
                .bind(job_type)
                .fetch_optional(&self.pool)
                .await?;

        Ok(enabled.unwrap_or(false))
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_queue_error_display() {
        let err = JobQueueError::Database("connection failed".to_string());
        assert!(err.to_string().contains("connection failed"));

        let err = JobQueueError::InvalidJobType("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
    }
}
