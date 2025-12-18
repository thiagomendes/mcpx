use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::services::crypto;
use crate::AppState;

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateCredentialRequest {
    pub credential_type: String,  // "api_key" or "bearer"
    pub value: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CredentialResponse {
    pub id: Uuid,
    pub server_id: Uuid,
    pub credential_type: String,
    pub name: Option<String>,
    pub masked_value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
struct Credential {
    id: Uuid,
    server_id: Uuid,
    credential_type: String,
    encrypted_value: String,
    name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// --- Handlers ---

/// List credentials for a server (masked values)
pub async fn list_credentials(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
) -> Result<Json<Vec<CredentialResponse>>, (StatusCode, String)> {
    // Get server and verify ownership
    let server = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM servers WHERE name = $1 AND user_id = $2"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    let credentials = sqlx::query_as::<_, Credential>(
        "SELECT id, server_id, credential_type, encrypted_value, name, created_at 
         FROM credentials WHERE server_id = $1"
    )
    .bind(&server)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<CredentialResponse> = credentials
        .into_iter()
        .map(|c| CredentialResponse {
            id: c.id,
            server_id: c.server_id,
            credential_type: c.credential_type,
            name: c.name,
            masked_value: mask_value(&decrypt_value(&c.encrypted_value, &state.config.encryption_key)),
            created_at: c.created_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Add a credential to a server
pub async fn create_credential(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
    Json(req): Json<CreateCredentialRequest>,
) -> Result<Json<CredentialResponse>, (StatusCode, String)> {
    // Validate credential type
    if req.credential_type != "api_key" && req.credential_type != "bearer" {
        return Err((StatusCode::BAD_REQUEST, "Invalid credential type. Use 'api_key' or 'bearer'".to_string()));
    }

    // Get server and verify ownership
    let server_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM servers WHERE name = $1 AND user_id = $2"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    // Encrypt the credential value
    let key = crypto::derive_key(&state.config.encryption_key);
    let encrypted = crypto::encrypt(&req.value, &key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Insert credential
    let credential = sqlx::query_as::<_, Credential>(
        "INSERT INTO credentials (server_id, credential_type, encrypted_value, name)
         VALUES ($1, $2, $3, $4)
         RETURNING id, server_id, credential_type, encrypted_value, name, created_at"
    )
    .bind(&server_id)
    .bind(&req.credential_type)
    .bind(&encrypted)
    .bind(&req.name)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update server auth_type
    let auth_type = match req.credential_type.as_str() {
        "api_key" => "api_key",
        "bearer" => "bearer",
        _ => "none",
    };
    
    sqlx::query("UPDATE servers SET auth_type = $1 WHERE id = $2")
        .bind(auth_type)
        .bind(&server_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CredentialResponse {
        id: credential.id,
        server_id: credential.server_id,
        credential_type: credential.credential_type,
        name: credential.name,
        masked_value: mask_value(&req.value),
        created_at: credential.created_at,
    }))
}

/// Delete a credential
pub async fn delete_credential(
    State(state): State<Arc<AppState>>,
    Path((server_name, credential_id)): Path<(String, Uuid)>,
    auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    // Get server and verify ownership
    let server_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM servers WHERE name = $1 AND user_id = $2"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    // Delete credential
    let result = sqlx::query(
        "DELETE FROM credentials WHERE id = $1 AND server_id = $2"
    )
    .bind(&credential_id)
    .bind(&server_id)
    .execute(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Credential not found".to_string()));
    }

    // Check if any credentials remain, if not reset auth_type
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM credentials WHERE server_id = $1")
        .bind(&server_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if count == 0 {
        sqlx::query("UPDATE servers SET auth_type = 'none' WHERE id = $1")
            .bind(&server_id)
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- Helpers ---

fn mask_value(value: &str) -> String {
    if value.len() <= 8 {
        "*".repeat(value.len())
    } else {
        format!("{}...{}", &value[..4], &value[value.len()-4..])
    }
}

fn decrypt_value(encrypted: &str, encryption_key: &str) -> String {
    let key = crypto::derive_key(encryption_key);
    crypto::decrypt(encrypted, &key).unwrap_or_else(|_| "***".to_string())
}
