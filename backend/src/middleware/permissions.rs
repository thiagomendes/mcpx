//! Permission System for Dashboard API
//!
//! SINGLE SOURCE OF TRUTH for all role-based permissions.
//! To change a permission, modify `allowed_roles()` only.

use axum::http::StatusCode;

/// All permissions in the system
#[derive(Debug, Clone, Copy)]
pub enum Permission {
    // Servers
    ServersRead,
    ServersWrite,

    // Gateways
    GatewaysRead,
    GatewaysWrite,

    // Personal Access Tokens
    TokensRead,
    TokensWrite,

    // Service Accounts
    ServiceAccountsRead,
    ServiceAccountsWrite,

    // Organization Members
    MembersRead,
    MembersInvite,
    MembersRemove,
    MembersChangeRole,

    // Settings
    SettingsRead,
    SettingsWrite,

    // Audit Logs
    AuditLogsRead,

    // Organization
    OrgDelete,
}

// Role constants for easy maintenance
const ALL_ROLES: &[&str] = &["owner", "admin", "member"];
const WRITE_ROLES: &[&str] = &["owner", "admin"];
const OWNER_ONLY: &[&str] = &["owner"];

/// Returns the roles allowed for a given permission.
///
/// MODIFY THIS FUNCTION TO CHANGE PERMISSIONS.
pub fn allowed_roles(permission: Permission) -> &'static [&'static str] {
    use Permission::*;

    match permission {
        // Read: all roles
        ServersRead | GatewaysRead | TokensRead | ServiceAccountsRead | MembersRead
        | SettingsRead => ALL_ROLES,

        // Write: owner + admin
        ServersWrite | GatewaysWrite | TokensWrite | ServiceAccountsWrite | MembersInvite
        | MembersRemove | SettingsWrite | AuditLogsRead => WRITE_ROLES,

        // Owner only
        MembersChangeRole | OrgDelete => OWNER_ONLY,
    }
}

/// Check if a role has a specific permission
pub fn has_permission(role: &str, permission: Permission) -> bool {
    allowed_roles(permission).contains(&role)
}

/// Require a permission, returning 403 if denied
pub fn require_permission(role: &str, permission: Permission) -> Result<(), (StatusCode, String)> {
    if has_permission(role, permission) {
        Ok(())
    } else {
        let action = format!("{:?}", permission);
        tracing::warn!("Permission denied: role={} action={}", role, action);
        Err((
            StatusCode::FORBIDDEN,
            format!(
                "Permission denied: {} requires {:?}",
                role,
                allowed_roles(permission)
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_has_all_permissions() {
        assert!(has_permission("owner", Permission::ServersRead));
        assert!(has_permission("owner", Permission::ServersWrite));
        assert!(has_permission("owner", Permission::MembersChangeRole));
        assert!(has_permission("owner", Permission::OrgDelete));
    }

    #[test]
    fn test_admin_cannot_change_roles() {
        assert!(has_permission("admin", Permission::ServersWrite));
        assert!(has_permission("admin", Permission::MembersInvite));
        assert!(!has_permission("admin", Permission::MembersChangeRole));
        assert!(!has_permission("admin", Permission::OrgDelete));
    }

    #[test]
    fn test_member_read_only() {
        assert!(has_permission("member", Permission::ServersRead));
        assert!(!has_permission("member", Permission::ServersWrite));
        assert!(!has_permission("member", Permission::MembersInvite));
    }
}
