# Test Script: Registering MCP Servers in mcpx

This script guides you through registering the example MCP servers in mcpx and testing different authentication methods.

## Prerequisites

1. Start all services (mcpx + example servers):
   ```bash
   # From project root
   docker compose up -d
   ```

2. Open mcpx dashboard: http://localhost:3000

**Important:** Since all services are on the same Docker network, use container names as URLs in mcpx.

---

## Test 1: Weather Server (API Key)

### Register in mcpx

1. Go to **Servers** page
2. Click **Add Server**
3. Fill in:
   - **Name**: `weather-local`
   - **URL**: `http://weather-mcp:8000/mcp`
   - **Transport**: Streamable HTTP
   - **Auth Type**: API Key
   - **Header Name**: `X-API-Key`
   - **API Key**: `weather-api-key-12345`
4. Click **Test Connection**
5. Should show tools: `get_weather`, `get_forecast`, etc.
6. Click **Save**

### Test via mcpx proxy

Copy the proxy URL and use it in your MCP client.

Expected behavior: mcpx adds the API Key header automatically.

---

## Test 2: Utilities Server (Bearer Token)

### Register in mcpx

1. Go to **Servers** page
2. Click **Add Server**
3. Fill in:
   - **Name**: `utilities-local`
   - **URL**: `http://utilities-mcp:8000/mcp`
   - **Transport**: Streamable HTTP
   - **Auth Type**: Bearer Token
   - **Token**: `utilities-token-secret-67890`
4. Click **Test Connection**
5. Should show tools: `encode_base64`, `decode_base64`, etc.
6. Click **Save**

### Test via mcpx proxy

Copy the proxy URL and use it in your MCP client.

Expected behavior: mcpx adds the Authorization header automatically.

---

## Test 3: Filesystem Server (OAuth Client Credentials)

### Register in mcpx

1. Go to **Servers** page
2. Click **Add Server**
3. Fill in:
   - **Name**: `filesystem-local`
   - **URL**: `http://filesystem-mcp:8000/mcp`
   - **Transport**: Streamable HTTP
   - **Auth Type**: OAuth (Client Credentials)
   - **Token URL**: `http://filesystem-mcp:8000/token`
   - **Client ID**: `filesystem-client`
   - **Client Secret**: `filesystem-secret-abc123`
4. Click **Test Connection**
5. Should show tools: `read_file`, `write_file`, etc.
6. Click **Save**

### Test via mcpx proxy

Copy the proxy URL and use it in your MCP client.

Expected behavior: mcpx obtains and refreshes tokens automatically.

---

## Test 4: Virtual Gateway (All Servers)

### Create Gateway

1. Go to **Gateways** page
2. Click **Add Gateway**
3. Fill in:
   - **Name**: `local-gateway`
4. Add servers:
   - `weather-local`
   - `utilities-local`
   - `filesystem-local`
5. Click **Save**

### Test via mcpx proxy

Use the gateway proxy URL. Should aggregate tools from all 3 servers with prefixes.

---

## Test 5: Tool Governance

### Configure Governance

1. Go to server **filesystem-local** details
2. Click **Governance** tab
3. Add to **Blocklist**: `delete_file`
4. Save

### Test

Call `delete_file` via the proxy. Should be blocked by mcpx.

---

## Cleanup

```bash
# From project root
docker compose down
```

---

## Troubleshooting

### Connection refused

If mcpx cannot connect to servers, ensure:
- Servers are running: `docker ps`
- Use container names (e.g., `weather-mcp:8000`) since all services are on the same Docker network

### Auth errors

Check server logs for details:
```bash
docker logs weather-mcp
docker logs utilities-mcp
docker logs filesystem-mcp
```
