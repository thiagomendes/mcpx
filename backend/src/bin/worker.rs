//! MCPX Worker
//!
//! Separate service for running background jobs with distributed locking.
//! Deploy as N replicas for high availability - PostgreSQL SKIP LOCKED
//! ensures each job runs exactly once.
//!
//! ## Usage
//!
//! ```bash
//! MCPX_WORKER_ID=worker-1 ./mcpx-worker
//! ```
//!
//! ## Adding New Jobs
//!
//! 1. Implement `JobHandler` trait in `jobs/handlers.rs`
//! 2. Register in `create_job_registry()`
//! 3. Add migration to insert into `job_schedules`

use mcpx_backend::{
    config::WorkerConfig,
    jobs::handlers::{create_job_registry, JobContext},
    services::{
        db::Database,
        job_queue::{JobQueue, PostgresJobQueue},
    },
};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mcpx_worker=info".parse().unwrap())
                .add_directive("sqlx=warn".parse().unwrap()),
        )
        .init();

    // Load .env
    dotenvy::dotenv().ok();

    // Load config
    let config = match WorkerConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to load config: {:?}", e);
            std::process::exit(1);
        }
    };

    // Connect to database
    let db: Database = match Database::new(&config.database_url).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = db.migrate().await {
        tracing::error!("Failed to run migrations: {}", e);
        std::process::exit(1);
    }

    // Worker ID (unique per instance)
    let worker_id = std::env::var("MCPX_WORKER_ID")
        .unwrap_or_else(|_| format!("worker-{}", &Uuid::new_v4().to_string()[..8]));

    tracing::info!("🔧 MCPX Worker {} starting...", worker_id);

    // Create job queue and registry
    let queue = PostgresJobQueue::new(db.pool.clone());
    let registry = create_job_registry();
    let context = JobContext::new(db.pool.clone(), config.encryption_key.clone());

    tracing::info!("Registered job handlers: {:?}", registry.job_types());

    // Main loop
    let poll_interval = Duration::from_secs(1);

    loop {
        match queue.claim_next(&worker_id).await {
            Ok(Some(job)) => {
                tracing::info!(
                    "[{}] Claimed job: {} (interval: {}s)",
                    worker_id,
                    job.job_type,
                    job.interval_seconds
                );

                let start = Instant::now();

                // Find and execute handler
                let result = if let Some(handler) = registry.get(&job.job_type) {
                    handler.execute(&context).await
                } else {
                    tracing::warn!("No handler for job type: {}", job.job_type);
                    Ok(()) // Skip unknown jobs
                };

                let duration = start.elapsed();

                match result {
                    Ok(()) => {
                        // Complete and reschedule
                        if let Err(e) = queue.complete(job.id, job.interval_seconds).await {
                            tracing::error!("Failed to complete job: {}", e);
                        }
                        tracing::info!(
                            "[{}] Completed: {} ({:.2}s)",
                            worker_id,
                            job.job_type,
                            duration.as_secs_f64()
                        );
                    }
                    Err(e) => {
                        // Release lock, will retry on next schedule
                        if let Err(release_err) = queue.release(job.id).await {
                            tracing::error!("Failed to release job: {}", release_err);
                        }
                        tracing::error!(
                            "[{}] Failed: {} - {} ({:.2}s)",
                            worker_id,
                            job.job_type,
                            e,
                            duration.as_secs_f64()
                        );
                    }
                }
            }
            Ok(None) => {
                // No jobs available, sleep briefly
                tokio::time::sleep(poll_interval).await;
            }
            Err(e) => {
                tracing::error!("Error claiming job: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
