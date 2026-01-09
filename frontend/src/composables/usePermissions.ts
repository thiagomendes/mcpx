/**
 * Permission System for Frontend
 * 
 * SINGLE SOURCE OF TRUTH - mirrors backend permissions.rs
 * To change a permission, modify PERMISSIONS only.
 */

import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'

// Role constants
const WRITE_ROLES = ['owner', 'admin'] as const
const OWNER_ONLY = ['owner'] as const

// Permission matrix - matches backend permissions.rs
export const PERMISSIONS = {
    // Write operations (create/update/delete)
    'servers:write': WRITE_ROLES,
    'gateways:write': WRITE_ROLES,
    'tokens:write': WRITE_ROLES,
    'service_accounts:write': WRITE_ROLES,

    // Member management
    'members:invite': WRITE_ROLES,
    'members:remove': WRITE_ROLES,
    'members:change_role': OWNER_ONLY,

    // Settings
    'settings:write': WRITE_ROLES,

    // Org management
    'org:delete': OWNER_ONLY,
} as const

export type Permission = keyof typeof PERMISSIONS

/**
 * Composable for checking user permissions
 */
export function usePermissions() {
    const authStore = useAuthStore()

    const userRole = computed(() => authStore.userRole)

    /**
     * Check if user has a specific permission
     */
    const can = (permission: Permission): boolean => {
        const allowedRoles = PERMISSIONS[permission]
        return (allowedRoles as readonly string[]).includes(userRole.value)
    }

    /**
     * Convenience: can user write (create/update/delete)?
     */
    const canWrite = computed(() => (WRITE_ROLES as readonly string[]).includes(userRole.value))

    /**
     * Convenience: is user an owner?
     */
    const isOwner = computed(() => userRole.value === 'owner')

    /**
     * Convenience: is user an admin (or owner)?
     */
    const isAdmin = computed(() => (WRITE_ROLES as readonly string[]).includes(userRole.value))

    /**
     * Convenience: is user a member (non-admin)?
     */
    const isMember = computed(() => userRole.value === 'member')

    return {
        userRole,
        can,
        canWrite,
        isOwner,
        isAdmin,
        isMember,
    }
}
