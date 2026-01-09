/**
 * Identity Provider abstraction for user authentication
 * 
 * This module defines the available identity providers for user login.
 * The actual OAuth flow happens on the backend - frontend only redirects.
 */

export type IdentityProviderType = 'google' | 'microsoft' | 'github';

export interface IdentityProviderConfig {
    id: IdentityProviderType;
    name: string;
    icon: string;
    enabled: boolean;
    loginPath: string;
    color: string;
    hoverColor: string;
}

const BACKEND_BASE = import.meta.env.VITE_API_URL || '/api';

export const IDENTITY_PROVIDERS: Record<IdentityProviderType, IdentityProviderConfig> = {
    google: {
        id: 'google',
        name: 'Google',
        icon: 'google',
        enabled: true,
        loginPath: `${BACKEND_BASE}/auth/google`,
        color: '#4285F4',
        hoverColor: '#3367D6',
    },
    github: {
        id: 'github',
        name: 'GitHub',
        icon: 'github',
        enabled: true, // Now enabled
        loginPath: `${BACKEND_BASE}/auth/github`,
        color: '#6e5494', // Purple for better contrast on dark theme
        hoverColor: '#5a3d7a',
    },
    microsoft: {
        id: 'microsoft',
        name: 'Microsoft',
        icon: 'microsoft',
        enabled: true, // Now enabled
        loginPath: `${BACKEND_BASE}/auth/microsoft`,
        color: '#00A4EF',
        hoverColor: '#0078D4',
    },
} as const;

/**
 * Get all enabled identity providers
 */
export function getEnabledProviders(): IdentityProviderConfig[] {
    return Object.values(IDENTITY_PROVIDERS).filter(p => p.enabled);
}

/**
 * Get a specific provider by ID
 */
export function getProvider(id: IdentityProviderType): IdentityProviderConfig | undefined {
    return IDENTITY_PROVIDERS[id];
}

/**
 * Redirect to provider login
 * The backend handles the OAuth flow and redirects back with token
 */
export function loginWithProvider(provider: IdentityProviderType): void {
    const config = IDENTITY_PROVIDERS[provider];
    if (!config || !config.enabled) {
        throw new Error(`Provider ${provider} is not enabled`);
    }
    window.location.href = config.loginPath;
}
