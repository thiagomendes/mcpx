/**
 * MCP OAuth Provider for mcpx frontend
 * 
 * Implements OAuthClientProvider interface from the TypeScript SDK
 * to handle OAuth flows entirely in the browser.
 * 
 * SECURITY: All sessionStorage is cleared after OAuth flow completes.
 * No tokens or sensitive data persisted in browser.
 */

// @ts-expect-error - SDK subpath exports work at runtime but not in TypeScript
import type { OAuthClientProvider } from '@modelcontextprotocol/sdk/client/auth';
// @ts-expect-error - SDK subpath exports work at runtime but not in TypeScript
import type { OAuthTokens, OAuthClientInformationMixed, OAuthClientMetadata } from '@modelcontextprotocol/sdk/shared/auth';

export interface McpxOAuthConfig {
  serverUrl: string;
  serverName: string;
  backendUrl: string;
}

const STATE_PREFIX = 'mcpx:';

/**
 * Custom OAuth Client Provider for mcpx
 * 
 * This provider handles the OAuth flow in the browser and sends
 * the obtained tokens to the backend for secure storage.
 */
export class McpxOAuthProvider implements OAuthClientProvider {
  private _tokens?: OAuthTokens;
  private _clientInformation?: OAuthClientInformationMixed;
  private _codeVerifier?: string;
  private _onComplete?: (tokens: OAuthTokens) => Promise<void>;

  constructor(
    private config: McpxOAuthConfig,
    onComplete?: (tokens: OAuthTokens) => Promise<void>
  ) {
    this._onComplete = onComplete;
  }

  get redirectUrl(): string | URL {

    return `${window.location.origin}/oauth/callback`;
  }

  get clientMetadata(): OAuthClientMetadata {
    return {
      redirect_uris: [this.redirectUrl.toString()],
      client_name: `mcpx-${this.config.serverName}`,
      grant_types: ['authorization_code', 'refresh_token'],
      response_types: ['code'],
      token_endpoint_auth_method: 'none' // Public client (SPA)
    };
  }

  /**
   * Generate custom state that encodes the serverName
   * This allows callback page to identify which server initiated the flow
   */
  state(): string {

    const random = crypto.randomUUID();
    return `${STATE_PREFIX}${this.config.serverName}:${random}`;
  }

  clientInformation(): OAuthClientInformationMixed | undefined {
    return this._clientInformation;
  }

  saveClientInformation(info: OAuthClientInformationMixed): void {
    this._clientInformation = info;

    sessionStorage.setItem(`mcpx_oauth_client_${this.config.serverName}`, JSON.stringify(info));
  }

  tokens(): OAuthTokens | undefined {
    return this._tokens;
  }

  async saveTokens(tokens: OAuthTokens): Promise<void> {
    this._tokens = tokens;


    if (this._onComplete) {
      await this._onComplete(tokens);
    }


    this.cleanup();
  }

  redirectToAuthorization(authorizationUrl: URL): void {

    sessionStorage.setItem(`mcpx_oauth_state_${this.config.serverName}`, JSON.stringify({
      serverUrl: this.config.serverUrl,
      serverName: this.config.serverName,
      clientInfo: this._clientInformation,
      codeVerifier: this._codeVerifier
    }));


    const width = 600;
    const height = 700;
    const left = window.screenX + (window.innerWidth - width) / 2;
    const top = window.screenY + (window.innerHeight - height) / 2;

    window.open(
      authorizationUrl.toString(),
      'mcpx_oauth',
      `width=${width},height=${height},left=${left},top=${top},popup=1`
    );
  }

  saveCodeVerifier(codeVerifier: string): void {
    this._codeVerifier = codeVerifier;

    sessionStorage.setItem(`mcpx_oauth_verifier_${this.config.serverName}`, codeVerifier);
  }

  codeVerifier(): string {
    if (this._codeVerifier) {
      return this._codeVerifier;
    }
    const stored = sessionStorage.getItem(`mcpx_oauth_verifier_${this.config.serverName}`);
    if (stored) {
      return stored;
    }
    throw new Error('No code verifier saved');
  }

  /**
   * Restore state from sessionStorage (for callback handling)
   */
  restoreState(serverName: string): boolean {
    const stateJson = sessionStorage.getItem(`mcpx_oauth_state_${serverName}`);
    if (!stateJson) return false;

    try {
      const state = JSON.parse(stateJson);
      this._clientInformation = state.clientInfo;
      this._codeVerifier = state.codeVerifier;
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Clean up ALL session storage - called after OAuth completes or fails
   * SECURITY: Ensures no tokens or sensitive data remain in browser
   */
  cleanup(): void {
    sessionStorage.removeItem(`mcpx_oauth_state_${this.config.serverName}`);
    sessionStorage.removeItem(`mcpx_oauth_verifier_${this.config.serverName}`);
    sessionStorage.removeItem(`mcpx_oauth_client_${this.config.serverName}`);
    this._tokens = undefined;
    this._clientInformation = undefined;
    this._codeVerifier = undefined;
  }

  /**
   * Parse serverName from state parameter
   * State format: mcpx:<serverName>:<random>
   */
  static parseServerNameFromState(state: string): string | null {
    if (!state.startsWith(STATE_PREFIX)) {
      return null;
    }
    const parts = state.slice(STATE_PREFIX.length).split(':');
    return parts[0] || null;
  }
}
