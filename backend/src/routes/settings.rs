//! Organization Settings Routes
//!
//! CRUD operations for org-level configuration settings.
//! Settings are stored as key-value pairs for flexibility.
//!
//! Access Control:
//! - GET: Any authenticated org member
//! - PUT: Only owner or admin

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::AppState;

// ============================================
// MODELS
// ============================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OrgSetting {
    pub id: Uuid,
    pub org_id: Uuid,
    pub setting_key: String,
    pub setting_value: String,
    pub setting_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SettingsWithMetadata {
    pub settings: Vec<SettingResponse>,
    pub can_edit: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: String,
}

// ============================================
// SETTING DEFINITIONS (validation bounds)
// ============================================

struct SettingDef {
    min: Option<i64>,
    max: Option<i64>,
}

fn get_setting_bounds(key: &str) -> Option<SettingDef> {
    match key {
        // Tokens (per-org configurable)
        "jwt_expiry_days" => Some(SettingDef {
            min: Some(1),
            max: Some(90),
        }),
        "m2m_token_expiry_hours" => Some(SettingDef {
            min: Some(1),
            max: Some(24),
        }),

        // Limits (per-org configurable)
        "max_servers_per_org" => Some(SettingDef {
            min: Some(1),
            max: Some(1000),
        }),
        "max_gateways_per_org" => Some(SettingDef {
            min: Some(1),
            max: Some(500),
        }),
        "max_pats_per_user" => Some(SettingDef {
            min: Some(1),
            max: Some(100),
        }),
        "max_service_accounts_per_org" => Some(SettingDef {
            min: Some(1),
            max: Some(100),
        }),

        _ => None,
    }
}

fn validate_setting_value(key: &str, value: &str) -> Result<(), (StatusCode, String)> {
    let bounds = get_setting_bounds(key);

    if let Some(def) = bounds {
        let parsed: i64 = value.parse().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                format!("Value for '{}' must be an integer", key),
            )
        })?;

        if let Some(min) = def.min {
            if parsed < min {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Value for '{}' must be at least {}", key, min),
                ));
            }
        }

        if let Some(max) = def.max {
            if parsed > max {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Value for '{}' must be at most {}", key, max),
                ));
            }
        }
    }

    Ok(())
}

// ============================================
// SQL QUERIES
// ============================================

const SQL_SELECT_ALL_SETTINGS: &str = r#"
    SELECT id, org_id, setting_key, setting_value, setting_type, description
    FROM org_settings
    WHERE org_id = $1
    ORDER BY setting_key
"#;

const SQL_SELECT_SETTING: &str = r#"
    SELECT id, org_id, setting_key, setting_value, setting_type, description
    FROM org_settings
    WHERE org_id = $1 AND setting_key = $2
"#;

const SQL_UPDATE_SETTING: &str = r#"
    UPDATE org_settings
    SET setting_value = $3, updated_at = NOW()
    WHERE org_id = $1 AND setting_key = $2
"#;

const SQL_GET_USER_ROLE: &str = r#"
    SELECT role FROM org_members WHERE org_id = $1 AND user_id = $2
"#;

// ============================================
// HANDLERS
// ============================================

/// GET /api/settings
/// Returns all settings for the current org with edit permission flag
pub async fn list_settings(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<SettingsWithMetadata>, (StatusCode, String)> {
    // Fetch all settings
    let settings: Vec<OrgSetting> = sqlx::query_as(SQL_SELECT_ALL_SETTINGS)
        .bind(auth_user.org_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Check user role
    let role: Option<(String,)> = sqlx::query_as(SQL_GET_USER_ROLE)
        .bind(auth_user.org_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let can_edit = role
        .map(|(r,)| r == "owner" || r == "admin")
        .unwrap_or(false);

    let response = SettingsWithMetadata {
        settings: settings
            .into_iter()
            .map(|s| SettingResponse {
                key: s.setting_key,
                value: s.setting_value,
                value_type: s.setting_type,
                description: s.description,
            })
            .collect(),
        can_edit,
    };

    Ok(Json(response))
}

/// GET /api/settings/:key
/// Returns a single setting value
pub async fn get_setting(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(key): Path<String>,
) -> Result<Json<SettingResponse>, (StatusCode, String)> {
    let setting: Option<OrgSetting> = sqlx::query_as(SQL_SELECT_SETTING)
        .bind(auth_user.org_id)
        .bind(&key)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    match setting {
        Some(s) => Ok(Json(SettingResponse {
            key: s.setting_key,
            value: s.setting_value,
            value_type: s.setting_type,
            description: s.description,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Setting '{}' not found", key),
        )),
    }
}

/// PUT /api/settings/:key
/// Update a setting value (owner/admin only)
pub async fn update_setting(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSettingRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Check user role - only owner or admin can modify
    let role: Option<(String,)> = sqlx::query_as(SQL_GET_USER_ROLE)
        .bind(auth_user.org_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let is_allowed = role
        .map(|(r,)| r == "owner" || r == "admin")
        .unwrap_or(false);

    if !is_allowed {
        return Err((
            StatusCode::FORBIDDEN,
            "Only owner or admin can modify settings".to_string(),
        ));
    }

    // Validate the value
    validate_setting_value(&key, &payload.value)?;

    // Check if setting exists
    let existing: Option<OrgSetting> = sqlx::query_as(SQL_SELECT_SETTING)
        .bind(auth_user.org_id)
        .bind(&key)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    if existing.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Setting '{}' not found", key),
        ));
    }

    // Update the setting
    sqlx::query(SQL_UPDATE_SETTING)
        .bind(auth_user.org_id)
        .bind(&key)
        .bind(&payload.value)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    tracing::info!(
        "Setting '{}' updated to '{}' by user {} in org {}",
        key,
        payload.value,
        auth_user.user_id,
        auth_user.org_id
    );

    Ok(StatusCode::OK)
}

// ============================================
// HELPER: Get setting value for internal use
// ============================================

pub async fn get_org_setting_value(
    db: &crate::Database,
    org_id: Uuid,
    key: &str,
    default: &str,
) -> String {
    let result: Option<(String,)> = sqlx::query_as(
        "SELECT setting_value FROM org_settings WHERE org_id = $1 AND setting_key = $2",
    )
    .bind(org_id)
    .bind(key)
    .fetch_optional(&db.pool)
    .await
    .ok()
    .flatten();

    result.map(|(v,)| v).unwrap_or_else(|| default.to_string())
}

// ============================================
// LIMIT STATUS ENDPOINT
// ============================================

#[derive(Debug, Serialize)]
pub struct LimitStatus {
    pub servers: LimitInfo,
    pub gateways: LimitInfo,
    pub pats: LimitInfo,
    pub service_accounts: LimitInfo,
}

#[derive(Debug, Serialize)]
pub struct LimitInfo {
    pub current: i64,
    pub max: i64,
    pub can_create: bool,
}

/// GET /api/limits - Get current resource counts vs limits
pub async fn get_limits(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<LimitStatus>, (StatusCode, String)> {
    let org_id = auth.org_id;
    let user_id = auth.user_id;

    // Get current counts
    let servers_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM servers WHERE org_id = $1")
        .bind(org_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let gateways_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM gateways WHERE org_id = $1")
        .bind(org_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let pats_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM personal_access_tokens WHERE user_id = $1 AND org_id = $2",
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::DATABASE_ERROR, e),
        )
    })?;

    let sas_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM service_accounts WHERE org_id = $1")
            .bind(org_id)
            .fetch_one(&state.db.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}: {}", error::DATABASE_ERROR, e),
                )
            })?;

    // Get limits from org_settings
    let max_servers = get_org_setting_value(&state.db, org_id, "max_servers_per_org", "100")
        .await
        .parse::<i64>()
        .unwrap_or(100);
    let max_gateways = get_org_setting_value(&state.db, org_id, "max_gateways_per_org", "50")
        .await
        .parse::<i64>()
        .unwrap_or(50);
    let max_pats = get_org_setting_value(&state.db, org_id, "max_pats_per_user", "10")
        .await
        .parse::<i64>()
        .unwrap_or(10);
    let max_sas = get_org_setting_value(&state.db, org_id, "max_service_accounts_per_org", "20")
        .await
        .parse::<i64>()
        .unwrap_or(20);

    Ok(Json(LimitStatus {
        servers: LimitInfo {
            current: servers_count.0,
            max: max_servers,
            can_create: servers_count.0 < max_servers,
        },
        gateways: LimitInfo {
            current: gateways_count.0,
            max: max_gateways,
            can_create: gateways_count.0 < max_gateways,
        },
        pats: LimitInfo {
            current: pats_count.0,
            max: max_pats,
            can_create: pats_count.0 < max_pats,
        },
        service_accounts: LimitInfo {
            current: sas_count.0,
            max: max_sas,
            can_create: sas_count.0 < max_sas,
        },
    }))
}
