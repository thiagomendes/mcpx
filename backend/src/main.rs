mod config;
mod routes;
mod models;
mod services;
mod middleware;

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
    
    // Use stderr for immediate output (no buffering)
    eprintln!("=== mcpx backend starting ===");
    std::io::stderr().flush().ok();
    
    // Load .env file
    dotenvy::dotenv().ok();
    eprintln!("1. Loaded .env");

    // Setup tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    println!("2. Tracing initialized");

    // Load config
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

    // Initialize database
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

    // Run migrations
    println!("6. Running migrations...");
    if let Err(e) = db.migrate().await {
        eprintln!("ERROR running migrations: {:?}", e);
        std::process::exit(1);
    }
    println!("7. Migrations complete!");

    // Create app state
    let state = Arc::new(AppState { config: config.clone(), db: db.clone() });

    // Start background health check job
    let health_check_interval: u64 = std::env::var("HEALTH_CHECK_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);
    
    services::health_check::start_health_check_job(
        db,
        config.encryption_key.clone(),
        health_check_interval
    ).await;
    println!("8. Health check job started ({}s interval)", health_check_interval);

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health check
        .route("/api/health", get(|| async { "OK" }))
        // Auth routes
        .route("/api/auth/google", get(routes::auth::google_login))
        .route("/api/auth/google/callback", get(routes::auth::google_callback))
        .route("/api/auth/me", get(routes::auth::get_current_user))
        .route("/api/auth/logout", post(routes::auth::logout))
        // Server routes
        .route("/api/servers", get(routes::servers::list_servers))
        .route("/api/servers", post(routes::servers::create_server))
        .route("/api/servers/:name", get(routes::servers::get_server))
        .route("/api/servers/:name", put(routes::servers::update_server))
        .route("/api/servers/:name", delete(routes::servers::delete_server))
        .route("/api/servers/:name/test", post(routes::servers::test_server))
        // Credentials routes
        .route("/api/servers/:name/credentials", get(routes::credentials::list_credentials))
        .route("/api/servers/:name/credentials", post(routes::credentials::create_credential))
        .route("/api/servers/:name/credentials/:id", delete(routes::credentials::delete_credential))
        // OAuth routes (frontend handles flow, backend stores tokens)
        .route("/api/servers/:name/oauth/store-tokens", post(routes::oauth::store_oauth_tokens))
        .route("/api/servers/:name/oauth/status", get(routes::oauth::oauth_status))
        .route("/api/servers/:name/oauth", delete(routes::oauth::revoke_oauth))
        // MCP Proxy route (public, no auth - uses user_id in path)
        .route("/mcp/:user_id/:server_name", post(routes::proxy::mcp_proxy))
        // Layers
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 mcpx backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
