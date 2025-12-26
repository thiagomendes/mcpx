#![allow(dead_code)]
pub mod error {
    pub const INVALID_TOKEN: &str = "Invalid token";
    pub const MISSING_AUTH_HEADER: &str = "Missing authorization header";
    pub const INVALID_AUTH_FORMAT: &str = "Invalid authorization format";
    pub const UNKNOWN_PROVIDER: &str = "Unknown provider";
    pub const TOKEN_EXCHANGE_FAILED: &str = "Token exchange failed";
    pub const USER_NOT_FOUND: &str = "User not found";
    pub const SERVER_NOT_FOUND: &str = "Server not found";
    pub const SERVER_DISABLED: &str = "Server is disabled";
    pub const INVALID_USER_ID: &str = "Invalid user ID";
    pub const DATABASE_ERROR: &str = "Database error";
    pub const JWT_ERROR: &str = "JWT error";
    pub const FAILED_TO_READ_BODY: &str = "Failed to read body";
    pub const INVALID_JSON: &str = "Invalid JSON";
    pub const GOVERNANCE_ERROR: &str = "Governance error";
    pub const AUTH_ERROR: &str = "Auth error";
    pub const PROXY_ERROR: &str = "Proxy error";
    pub const CREDENTIAL_NOT_FOUND: &str = "Credential not found";
    pub const INVALID_CREDENTIAL_TYPE: &str = "Invalid credential type. Use 'api_key' or 'bearer'";
    pub const ENCRYPTION_ERROR: &str = "Encryption error";
    pub const SERVER_EXISTS: &str = "Server with this name already exists";
    pub const INVALID_PREFIX_FORMAT: &str = "Prefix must be alphanumeric with underscores only";
    pub const MUTUALLY_EXCLUSIVE: &str = "Cannot specify both allowed_tools and denied_tools";
    pub const INVALID_SERVER_NAME: &str = "Invalid server name. Use only letters, numbers, hyphens, and underscores";
    
    pub const CIPHER_CREATE_FAILED: &str = "Failed to create cipher";
    pub const ENCRYPTION_FAILED: &str = "Encryption failed";
    pub const BASE64_DECODE_FAILED: &str = "Base64 decode failed";
    pub const INVALID_ENCRYPTED_DATA: &str = "Invalid encrypted data: too short";
    pub const DECRYPTION_FAILED: &str = "Decryption failed";
    pub const INVALID_UTF8: &str = "Invalid UTF-8";
}

pub mod oauth {
    pub const MICROSOFT_NOT_CONFIGURED: &str = "Microsoft OAuth not configured";
    pub const GITHUB_NOT_CONFIGURED: &str = "GitHub OAuth not configured";
}
