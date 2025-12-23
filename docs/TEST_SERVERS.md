# MCP Servers - Test Configuration

Servidores para testar o MCPX em ambiente de desenvolvimento.

## Servidores Recomendados

### 1. DeepWiki (No Auth - Público)

**Ideal para testes iniciais - sem autenticação necessária**

| Campo | Valor |
|-------|-------|
| **name** | `deepwiki` |
| **url** | `https://mcp.deepwiki.com/mcp` |
| **auth_type** | `none` |
| **transport** | Streamable HTTP |

```json
{
  "name": "deepwiki",
  "url": "https://mcp.deepwiki.com/mcp",
  "auth_type": "none"
}
```

---

### 2. Example Remote Server (OAuth - Anthropic)

**Servidor de exemplo oficial do MCP - OAuth auto-discovery**

| Campo | Valor |
|-------|-------|
| **name** | `example` |
| **url** | `https://example-server.modelcontextprotocol.io/mcp` |
| **auth_type** | `oauth_auto` |
| **transport** | Streamable HTTP |
| **governance prefix** | `ex` |

```json
{
  "name": "example",
  "url": "https://example-server.modelcontextprotocol.io/mcp",
  "auth_type": "oauth_auto"
}
```

---

### 3. Cloudflare Docs (No Auth - Público)

| Campo | Valor |
|-------|-------|
| **name** | `cloudflare-docs` |
| **url** | `https://docs.mcp.cloudflare.com/mcp` |
| **auth_type** | `none` |
| **transport** | Streamable HTTP |

```json
{
  "name": "cloudflare-docs",
  "url": "https://docs.mcp.cloudflare.com/mcp",
  "auth_type": "none"
}
```

---

### 3. Cloudflare Workers (OAuth)

| Campo | Valor |
|-------|-------|
| **name** | `cloudflare` |
| **url** | `https://bindings.mcp.cloudflare.com/mcp` |
| **auth_type** | `oauth_auto` |
| **transport** | Streamable HTTP |
| **governance prefix** | `cf` |

```json
{
  "name": "cloudflare",
  "url": "https://bindings.mcp.cloudflare.com/mcp",
  "auth_type": "oauth_auto"
}
```

---

### 4. Exa Search (API Key)

| Campo | Valor |
|-------|-------|
| **name** | `exa` |
| **url** | `https://mcp.exa.ai/mcp` |
| **auth_type** | `bearer` |
| **transport** | Streamable HTTP |

```json
{
  "name": "exa",
  "url": "https://mcp.exa.ai/mcp",
  "auth_type": "bearer"
}
```
*Requer: Criar credential `bearer_token` com API key da Exa*

---

## Gateway de Teste

```json
{
  "name": "Dev Gateway",
  "slug": "dev",
  "servers": ["deepwiki", "cloudflare-docs"]
}
```

---

## Referência Rápida

| Servidor | URL | Auth | Prefix |
|----------|-----|------|--------|
| DeepWiki | `https://mcp.deepwiki.com/mcp` | none | - |
| Example | `https://example-server.modelcontextprotocol.io/mcp` | oauth | `ex` |
| CF Docs | `https://docs.mcp.cloudflare.com/mcp` | none | - |
| CF Workers | `https://bindings.mcp.cloudflare.com/mcp` | oauth | `cf` |
| Exa | `https://mcp.exa.ai/mcp` | bearer | - |
| Tavily | `https://mcp.tavily.com/mcp` | api_key | - |
| Neon | `https://mcp.neon.tech/mcp` | oauth/bearer | - |
| Linear | `https://mcp.linear.app/mcp` | oauth | - |
| Sentry | `https://mcp.sentry.dev/mcp` | oauth | - |
