//! MCPX Web Server
//!
//! HTTP API for the MCPX dashboard and MCP proxy.
//! Background jobs run in the separate worker service.

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use mcpx_backend::{config::Config, routes, services::db::Database, AppState};

#[tokio::main]
async fn main() {
    use std::io::Write;

    eprintln!("=== mcpx web server starting ===");
    std::io::stderr().flush().ok();

    dotenvy::dotenv().ok();
    eprintln!("1. Loaded .env");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
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

    let state = AppState::new(config.clone(), db.clone());

    // NOTE: Background jobs now run in separate worker service
    println!("8. Background jobs delegated to worker service");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        // OAuth routes - generic handler for all providers
        .route("/api/auth/:provider", get(routes::auth::oauth_login))
        .route(
            "/api/auth/:provider/callback",
            get(routes::auth::oauth_callback),
        )
        // Legacy Google routes (redirect to generic)
        .route("/api/auth/google", get(routes::auth::google_login))
        .route(
            "/api/auth/google/callback",
            get(routes::auth::google_callback),
        )
        // User endpoints
        .route("/api/auth/me", get(routes::auth::get_current_user))
        .route(
            "/api/auth/switch-org/:org_id",
            post(routes::auth::switch_org),
        )
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/servers", get(routes::servers::list_servers))
        .route("/api/servers", post(routes::servers::create_server))
        .route("/api/servers/:name", get(routes::servers::get_server))
        .route("/api/servers/:name", put(routes::servers::update_server))
        .route("/api/servers/:name", delete(routes::servers::delete_server))
        .route(
            "/api/servers/:name/test",
            post(routes::servers::test_server),
        )
        .route(
            "/api/servers/:name/credentials",
            get(routes::credentials::list_credentials),
        )
        .route(
            "/api/servers/:name/credentials",
            post(routes::credentials::create_credential),
        )
        .route(
            "/api/servers/:name/credentials/:id",
            delete(routes::credentials::delete_credential),
        )
        .route(
            "/api/servers/:name/oauth/store-tokens",
            post(routes::oauth::store_oauth_tokens),
        )
        .route(
            "/api/servers/:name/oauth/status",
            get(routes::oauth::oauth_status),
        )
        .route(
            "/api/servers/:name/oauth",
            delete(routes::oauth::revoke_oauth),
        )
        .route(
            "/api/servers/:name/governance",
            get(routes::governance::get_governance),
        )
        .route(
            "/api/servers/:name/governance",
            post(routes::governance::set_governance),
        )
        .route(
            "/api/servers/:name/governance",
            delete(routes::governance::delete_governance),
        )
        // Gateways
        .route("/api/gateways", get(routes::gateways::list_gateways))
        .route("/api/gateways", post(routes::gateways::create_gateway))
        .route("/api/gateways/:slug", get(routes::gateways::get_gateway))
        .route("/api/gateways/:slug", put(routes::gateways::update_gateway))
        .route(
            "/api/gateways/:slug",
            delete(routes::gateways::delete_gateway),
        )
        .route(
            "/api/gateways/:slug/servers",
            post(routes::gateways::add_server_to_gateway),
        )
        .route(
            "/api/gateways/:slug/servers/:name",
            delete(routes::gateways::remove_server_from_gateway),
        )
        .route(
            "/api/gateways/:slug/tools",
            get(routes::gateways::list_gateway_tools),
        )
        // Metrics
        .route("/api/metrics/today", get(routes::metrics::get_today))
        .route("/api/metrics/hourly", get(routes::metrics::get_hourly))
        .route(
            "/api/metrics/by-target",
            get(routes::metrics::get_by_target),
        )
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
        .route(
            "/api/alerts/:id/acknowledge",
            post(routes::alerts::acknowledge_alert),
        )
        // Organizations
        .route("/api/orgs", post(routes::orgs::create_org))
        .route("/api/orgs/:id", delete(routes::orgs::delete_org))
        .route("/api/orgs/members", get(routes::orgs::list_members))
        .route(
            "/api/orgs/members/:user_id",
            delete(routes::orgs::remove_member),
        )
        .route(
            "/api/orgs/members/invite",
            post(routes::orgs::invite_member),
        )
        .route(
            "/api/orgs/join/:token",
            get(routes::orgs::get_invite_details).post(routes::orgs::join_org),
        )
        // Account management
        .route("/api/account", delete(routes::orgs::delete_account))
        .route(
            "/api/account/deletion-preview",
            get(routes::orgs::preview_account_deletion),
        )
        // Personal Access Tokens
        .route("/api/tokens", get(routes::pat::list_tokens))
        .route("/api/tokens", post(routes::pat::create_token))
        .route("/api/tokens/:id", delete(routes::pat::delete_token))
        // Service Accounts (M2M)
        .route(
            "/api/service-accounts",
            get(routes::service_accounts::list_service_accounts),
        )
        .route(
            "/api/service-accounts",
            post(routes::service_accounts::create_service_account),
        )
        .route(
            "/api/service-accounts/:id",
            get(routes::service_accounts::get_service_account),
        )
        .route(
            "/api/service-accounts/:id",
            put(routes::service_accounts::update_service_account),
        )
        .route(
            "/api/service-accounts/:id",
            delete(routes::service_accounts::delete_service_account),
        )
        .route("/api/auth/token", post(routes::auth::oauth_token))
        // Organization Settings
        .route("/api/settings", get(routes::settings::list_settings))
        .route("/api/settings/:key", get(routes::settings::get_setting))
        .route("/api/settings/:key", put(routes::settings::update_setting))
        .route("/api/limits", get(routes::settings::get_limits))
        // MCP Proxy - now uses org_slug instead of user_id
        .route(
            "/mcp/:org_slug/:server_name",
            post(routes::proxy::mcp_proxy),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 mcpx web server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
