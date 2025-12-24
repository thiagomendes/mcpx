# MCP Servers Examples

Example MCP servers for testing different authentication methods with mcpx.

## Servers

| Server | Port | Auth Method | Credentials |
|--------|------|-------------|-------------|
| Weather | 8001 | API Key | `X-API-Key: weather-api-key-12345` |
| Utilities | 8002 | Bearer Token | `Authorization: Bearer utilities-token-secret-67890` |
| Filesystem | 8003 | OAuth Client Credentials | See below |

### Filesystem OAuth Credentials

```
Client ID: filesystem-client
Client Secret: filesystem-secret-abc123
Token Endpoint: POST http://localhost:8003/token
```

## Quick Start

The example servers are defined in the root `docker-compose.yml` file.

```bash
# From project root
docker compose up -d

# Check servers are running
curl http://localhost:8001/health
curl http://localhost:8002/health
curl http://localhost:8003/health
```

When registering servers in mcpx, use the container names since they are on the same network:
- Weather: `http://weather-mcp:8000/mcp`
- Utilities: `http://utilities-mcp:8000/mcp`
- Filesystem: `http://filesystem-mcp:8000/mcp`

## Testing with mcpx

See [TEST_SCRIPT.md](./TEST_SCRIPT.md) for step-by-step testing instructions.

## Auth Examples

### Weather (API Key)

```bash
curl -X POST http://localhost:8001/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: weather-api-key-12345" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
```

### Utilities (Bearer Token)

```bash
curl -X POST http://localhost:8002/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer utilities-token-secret-67890" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
```

### Filesystem (OAuth Client Credentials)

```bash
# Step 1: Get token
TOKEN=$(curl -s -X POST http://localhost:8003/token \
  -H "Content-Type: application/json" \
  -d '{"client_id":"filesystem-client","client_secret":"filesystem-secret-abc123","grant_type":"client_credentials"}' \
  | jq -r '.access_token')

# Step 2: Use token
curl -X POST http://localhost:8003/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
```

## Stop Servers

```bash
docker compose down
```
