mod config;
mod routes;
mod models;
mod services;
mod middleware;
mod messages;
mod jobs;

use axum::{
    routing::{get, post, delete, put},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

use config::Config;
use services::db::Database;

pub struct AppState {
    pub config: Config,
    pub db: Database,
}

#[tokio::main]
async fn main() {
    use std::io::Write;
    

    eprintln!("=== mcpx backend starting ===");
    std::io::stderr().flush().ok();
    

    dotenvy::dotenv().ok();
    eprintln!("1. Loaded .env");


    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    println!("2. Tracing initialized");


    let config = match Config::from_env() {
        Ok(c) => {
            println!("3. Config loaded successfully");
            c
        }
        Err(e) => {
            eprintln!("ERROR loading config: {:?}", e);
            std::process::exit(1);
        }
    };
    let port = config.backend_port;


    println!("4. Connecting to database: {}", &config.database_url);
    let db = match Database::new(&config.database_url).await {
        Ok(d) => {
            println!("5. Database connected!");
            d
        }
        Err(e) => {
            eprintln!("ERROR connecting to database: {:?}", e);
            std::process::exit(1);
        }
    };


    println!("6. Running migrations...");
    if let Err(e) = db.migrate().await {
        eprintln!("ERROR running migrations: {:?}", e);
        std::process::exit(1);
    }
    println!("7. Migrations complete!");


    let state = Arc::new(AppState { config: config.clone(), db: db.clone() });


    let health_check_interval: u64 = std::env::var("HEALTH_CHECK_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);
    
    services::health_check::start_health_check_job(
        db.clone(),
        config.encryption_key.clone(),
        health_check_interval
    ).await;
    println!("8. Health check job started ({}s interval)", health_check_interval);

    // Alert Evaluator Job (designed for future extraction to separate service)
    let alert_eval_interval: u64 = std::env::var("ALERT_EVAL_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);
    
    let alert_pool = db.pool.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(alert_eval_interval)).await;
            match jobs::alert_evaluator::run_evaluation_cycle(&alert_pool).await {
                Ok(results) => {
                    if !results.is_empty() {
                        tracing::debug!("Alert evaluation completed: {} rules checked", results.len());
                    }
                }
                Err(e) => {
                    tracing::error!("Alert evaluation failed: {}", e);
                }
            }
        }
    });
    println!("9. Alert evaluator job started ({}s interval)", alert_eval_interval);


    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);


    let app = Router::new()
    
        .route("/api/health", get(|| async { "OK" }))
    
        // OAuth routes - generic handler for all providers
        .route("/api/auth/:provider", get(routes::auth::oauth_login))
        .route("/api/auth/:provider/callback", get(routes::auth::oauth_callback))
        // Legacy Google routes (redirect to generic)
        .route("/api/auth/google", get(routes::auth::google_login))
        .route("/api/auth/google/callback", get(routes::auth::google_callback))
        // User endpoints
        .route("/api/auth/me", get(routes::auth::get_current_user))
        .route("/api/auth/switch-org/:org_id", post(routes::auth::switch_org))
        .route("/api/auth/logout", post(routes::auth::logout))
    
        .route("/api/servers", get(routes::servers::list_servers))
        .route("/api/servers", post(routes::servers::create_server))
        .route("/api/servers/:name", get(routes::servers::get_server))
        .route("/api/servers/:name", put(routes::servers::update_server))
        .route("/api/servers/:name", delete(routes::servers::delete_server))
        .route("/api/servers/:name/test", post(routes::servers::test_server))
    
        .route("/api/servers/:name/credentials", get(routes::credentials::list_credentials))
        .route("/api/servers/:name/credentials", post(routes::credentials::create_credential))
        .route("/api/servers/:name/credentials/:id", delete(routes::credentials::delete_credential))
    
        .route("/api/servers/:name/oauth/store-tokens", post(routes::oauth::store_oauth_tokens))
        .route("/api/servers/:name/oauth/status", get(routes::oauth::oauth_status))
        .route("/api/servers/:name/oauth", delete(routes::oauth::revoke_oauth))
    
        .route("/api/servers/:name/governance", get(routes::governance::get_governance))
        .route("/api/servers/:name/governance", post(routes::governance::set_governance))
        .route("/api/servers/:name/governance", delete(routes::governance::delete_governance))
    
        // Gateways
        .route("/api/gateways", get(routes::gateways::list_gateways))
        .route("/api/gateways", post(routes::gateways::create_gateway))
        .route("/api/gateways/:slug", get(routes::gateways::get_gateway))
        .route("/api/gateways/:slug", put(routes::gateways::update_gateway))
        .route("/api/gateways/:slug", delete(routes::gateways::delete_gateway))
        .route("/api/gateways/:slug/servers", post(routes::gateways::add_server_to_gateway))
        .route("/api/gateways/:slug/servers/:name", delete(routes::gateways::remove_server_from_gateway))
    
        // Metrics
        .route("/api/metrics/today", get(routes::metrics::get_today))
        .route("/api/metrics/hourly", get(routes::metrics::get_hourly))
        .route("/api/metrics/by-target", get(routes::metrics::get_by_target))
        .route("/api/metrics/query", post(routes::metrics::query))
        .route("/api/metrics/summary", get(routes::metrics::get_summary))
    
        // Audit Logs
        .route("/api/audit", get(routes::audit::list_audit_logs))
        .route("/api/audit/:id", get(routes::audit::get_audit_log))
    
        // Alerts
        .route("/api/alerts", get(routes::alerts::list_rules))
        .route("/api/alerts", post(routes::alerts::create_rule))
        .route("/api/alerts/active", get(routes::alerts::get_active_alerts))
        .route("/api/alerts/history", get(routes::alerts::get_history))
        .route("/api/alerts/:id", get(routes::alerts::get_rule))
        .route("/api/alerts/:id", put(routes::alerts::update_rule))
        .route("/api/alerts/:id", delete(routes::alerts::delete_rule))
        .route("/api/alerts/:id/acknowledge", post(routes::alerts::acknowledge_alert))
    
        // Organizations
        .route("/api/orgs", post(routes::orgs::create_org))
        .route("/api/orgs/:id", delete(routes::orgs::delete_org))
        .route("/api/orgs/members", get(routes::orgs::list_members))
        .route("/api/orgs/members/:user_id", delete(routes::orgs::remove_member))
        .route("/api/orgs/members/invite", post(routes::orgs::invite_member))
        .route("/api/orgs/join/:token", get(routes::orgs::get_invite_details).post(routes::orgs::join_org))
    
        // Personal Access Tokens
        .route("/api/tokens", get(routes::pat::list_tokens))
        .route("/api/tokens", post(routes::pat::create_token))
        .route("/api/tokens/:id", delete(routes::pat::delete_token))
    
        // MCP Proxy - now uses org_slug instead of user_id
        .route("/mcp/:org_slug/:server_name", post(routes::proxy::mcp_proxy))
    
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);


    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 mcpx backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
