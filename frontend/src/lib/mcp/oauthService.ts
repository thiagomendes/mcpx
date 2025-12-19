/**
 * MCP OAuth Service for mcpx frontend
 * 
 * High-level service that handles the OAuth authorization flow
 * using the MCP TypeScript SDK.
 * 
 * SECURITY: All browser storage is cleaned up after OAuth completes.
 */

import { auth } from '@modelcontextprotocol/sdk/client/auth';
import type { OAuthTokens } from '@modelcontextprotocol/sdk/shared/auth';
import { McpxOAuthProvider } from './oauthProvider';
import api from '@/api/client';

export interface OAuthResult {
    success: boolean;
    error?: string;
}

/**
 * Start OAuth authorization flow for an MCP server
 * 
 * @param serverName - Name of the server in mcpx
 * @param serverUrl - URL of the upstream MCP server
 * @returns Promise that resolves when OAuth flow completes
 */
export async function startOAuthFlow(
    serverName: string,
    serverUrl: string
): Promise<OAuthResult> {
    const provider = new McpxOAuthProvider(
        {
            serverUrl,
            serverName,
            backendUrl: ''
        },
        async (tokens: OAuthTokens) => {
            // Send tokens to backend for secure storage
            await storeTokensInBackend(serverName, tokens);
        }
    );

    try {
        // Use SDK to start authorization
        // SDK signature: auth(provider, { serverUrl, ... })
        const result = await auth(provider, { serverUrl });

        // Cleanup browser storage regardless of outcome
        provider.cleanup();

        if (result === 'AUTHORIZED') {
            return { success: true };
        } else if (result === 'REDIRECT') {
            // User was redirected to authorization server
            // Callback will be handled by handleOAuthCallback
            return { success: true };
        } else {
            return { success: false, error: 'Authorization was cancelled or failed' };
        }
    } catch (error) {
        // Cleanup browser storage on error
        provider.cleanup();

        console.error('OAuth flow error:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error'
        };
    }
}

/**
 * Handle OAuth callback (when popup redirects back)
 * This should be called from the OAuth callback page
 */
export async function handleOAuthCallback(
    serverName: string,
    code: string,
    _state: string  // State is validated by SDK internally
): Promise<OAuthResult> {
    // Get the stored server URL from sessionStorage
    const stateJson = sessionStorage.getItem(`mcpx_oauth_state_${serverName}`);
    if (!stateJson) {
        return { success: false, error: 'OAuth state not found' };
    }

    let savedState: { serverUrl: string; serverName: string };
    try {
        savedState = JSON.parse(stateJson);
    } catch {
        return { success: false, error: 'Invalid OAuth state' };
    }

    const provider = new McpxOAuthProvider(
        {
            serverUrl: savedState.serverUrl,
            serverName,
            backendUrl: ''
        },
        async (tokens: OAuthTokens) => {
            await storeTokensInBackend(serverName, tokens);
        }
    );

    try {
        if (!provider.restoreState(serverName)) {
            return { success: false, error: 'Could not restore OAuth state' };
        }

        // Exchange code for tokens using SDK
        // SDK signature: auth(provider, { serverUrl, authorizationCode })
        const result = await auth(provider, {
            serverUrl: savedState.serverUrl,
            authorizationCode: code
        });

        // Cleanup is done automatically in saveTokens(), but call again for safety
        provider.cleanup();

        if (result === 'AUTHORIZED') {
            return { success: true };
        } else {
            return { success: false, error: 'Token exchange failed' };
        }
    } catch (error) {
        // Cleanup browser storage on error
        provider.cleanup();

        console.error('OAuth callback error:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error'
        };
    }
}

/**
 * Store tokens in backend for secure persistence
 */
async function storeTokensInBackend(serverName: string, tokens: OAuthTokens): Promise<void> {
    await api.post(`/servers/${serverName}/oauth/store-tokens`, {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
        scope: tokens.scope
    });
}
