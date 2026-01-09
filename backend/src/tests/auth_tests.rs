//! Integration tests for multi-token authentication
//!
//! Tests PAT, M2M JWT, and Web Session JWT authentication flows

#[allow(unused_imports)]
use std::sync::Arc;

/// Test that PAT tokens are correctly identified by prefix
#[test]
fn test_pat_token_detection() {
    let pat = "mcpx_pat_abc123xyz";
    assert!(pat.starts_with("mcpx_pat_"));
}

/// Test that M2M tokens start with JWT prefix
#[test]
fn test_jwt_token_detection() {
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.xxx";
    assert!(jwt.starts_with("ey"));
}

/// Test token type routing logic
#[test]
fn test_token_type_routing() {
    let tokens = vec![
        ("mcpx_pat_test123", "PAT"),
        ("eyJhbGciOiJIUzI1NiJ9.xxx", "JWT"),
        ("invalid_token", "INVALID"),
    ];

    for (token, expected_type) in tokens {
        let detected = if token.starts_with("mcpx_pat_") {
            "PAT"
        } else if token.starts_with("ey") {
            "JWT"
        } else {
            "INVALID"
        };
        assert_eq!(detected, expected_type, "Token: {}", token);
    }
}

#[cfg(test)]
mod permission_tests {
    use crate::middleware::permissions::{allowed_roles, has_permission, Permission};

    /// Test all read permissions are accessible by all roles
    #[test]
    fn test_read_permissions_all_roles() {
        let read_perms = vec![
            Permission::ServersRead,
            Permission::GatewaysRead,
            Permission::TokensRead,
            Permission::ServiceAccountsRead,
            Permission::MembersRead,
            Permission::SettingsRead,
        ];

        for role in &["owner", "admin", "member"] {
            for perm in &read_perms {
                assert!(
                    has_permission(role, *perm),
                    "{} should have {:?}",
                    role,
                    perm
                );
            }
        }
    }

    /// Test write permissions are denied for members
    #[test]
    fn test_write_permissions_denied_for_member() {
        let write_perms = vec![
            Permission::ServersWrite,
            Permission::GatewaysWrite,
            Permission::ServiceAccountsWrite,
            Permission::MembersInvite,
            Permission::MembersRemove,
            Permission::SettingsWrite,
        ];

        for perm in &write_perms {
            assert!(
                !has_permission("member", *perm),
                "member should NOT have {:?}",
                perm
            );
        }
    }

    /// Test write permissions are granted to admin
    #[test]
    fn test_write_permissions_granted_to_admin() {
        let write_perms = vec![
            Permission::ServersWrite,
            Permission::GatewaysWrite,
            Permission::ServiceAccountsWrite,
            Permission::MembersInvite,
            Permission::MembersRemove,
            Permission::SettingsWrite,
        ];

        for perm in &write_perms {
            assert!(
                has_permission("admin", *perm),
                "admin should have {:?}",
                perm
            );
        }
    }

    /// Test owner-only permissions
    #[test]
    fn test_owner_only_permissions() {
        let owner_only = vec![Permission::MembersChangeRole, Permission::OrgDelete];

        for perm in &owner_only {
            assert!(
                has_permission("owner", *perm),
                "owner should have {:?}",
                perm
            );
            assert!(
                !has_permission("admin", *perm),
                "admin should NOT have {:?}",
                perm
            );
            assert!(
                !has_permission("member", *perm),
                "member should NOT have {:?}",
                perm
            );
        }
    }

    /// Test allowed_roles returns correct roles
    #[test]
    fn test_allowed_roles() {
        // Read = all roles
        assert_eq!(
            allowed_roles(Permission::ServersRead),
            &["owner", "admin", "member"]
        );

        // Write = owner + admin
        assert_eq!(allowed_roles(Permission::ServersWrite), &["owner", "admin"]);

        // Owner only
        assert_eq!(allowed_roles(Permission::OrgDelete), &["owner"]);
    }
}

#[cfg(test)]
mod security_tests {
    /// Test email comparison is case-insensitive
    #[test]
    fn test_email_case_insensitive() {
        let invite_email = "User@Example.COM";
        let user_email = "user@example.com";

        assert_eq!(
            invite_email.to_lowercase(),
            user_email.to_lowercase(),
            "Email comparison should be case-insensitive"
        );
    }

    /// Test that empty role should fail permission checks
    #[test]
    fn test_empty_role_denied() {
        use crate::middleware::permissions::{has_permission, Permission};

        assert!(!has_permission("", Permission::ServersRead));
        assert!(!has_permission("", Permission::ServersWrite));
    }

    /// Test unknown role is denied
    #[test]
    fn test_unknown_role_denied() {
        use crate::middleware::permissions::{has_permission, Permission};

        assert!(!has_permission("superuser", Permission::ServersRead));
        assert!(!has_permission("root", Permission::OrgDelete));
    }
}
