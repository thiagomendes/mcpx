/**
 * Frontend constants and error messages
 */

export const STORAGE_KEYS = {
    TOKEN: 'token',
    USER: 'user',
    OAUTH_STATE: 'oauth_state',
    OAUTH_SERVER: 'oauth_server_name',
    OAUTH_CODE_VERIFIER: 'oauth_code_verifier',
} as const;

export const ROUTES = {
    LOGIN: '/login',
    DASHBOARD: '/dashboard',
    SERVERS: '/servers',
} as const;

export const API_ENDPOINTS = {
    AUTH_ME: '/auth/me',
    SERVERS: '/servers',
    GOVERNANCE: (serverName: string) => `/servers/${serverName}/governance`,
    CREDENTIALS: (serverName: string) => `/servers/${serverName}/credentials`,
    OAUTH_TOKENS: (serverName: string) => `/servers/${serverName}/oauth/tokens`,
    OAUTH_STATUS: (serverName: string) => `/servers/${serverName}/oauth/status`,
} as const;

export const ERROR_MESSAGES = {
    FETCH_SERVERS_FAILED: 'Failed to fetch servers',
    FETCH_GOVERNANCE_FAILED: 'Failed to fetch governance config',
    SAVE_GOVERNANCE_FAILED: 'Failed to save governance config',
    FETCH_TOOLS_FAILED: 'Failed to fetch available tools',
    OAUTH_FLOW_ERROR: 'OAuth flow error',
    OAUTH_CALLBACK_ERROR: 'OAuth callback error',
    INVALID_SERVER_NAME: 'Invalid server name',
    SERVER_NOT_FOUND: 'Server not found',
} as const;

export const GOVERNANCE_MODES = {
    NONE: 'none',
    ALLOWLIST: 'allowlist',
    DENYLIST: 'denylist',
} as const;
