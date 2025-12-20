# MCPX - Estado do Projeto e Próximos Passos

**Última atualização:** 2025-12-20 19:47
**Commit mais recente:** c90f5e1 (feat: Server UX v2 - Health Check System & OAuth Flow Improvements)

---

## ⚠️ EM PROGRESSO - NÃO VALIDADO

### MCP Proxy Layer (`/mcp/:user_id/:server_name`)

**Arquivo:** `backend/src/routes/proxy.rs`

**Status:** Implementado mas **NÃO VALIDADO** completamente.

**O que foi feito:**
- Criado handler `mcp_proxy` em `proxy.rs`
- Adicionado rota em `main.rs`
- Adicionado `Accept: application/json, text/event-stream` header
- Adicionado forwarding do header `Mcp-Session-Id`
- Injeção de auth headers (API Key, Bearer, OAuth)

**Testes parciais:**
- ✅ `initialize` funcionou (resposta válida do DeepWiki)
- ❌ `tools/list` falhou (requer Mcp-Session-Id)

**Próximo passo imediato:**
1. Rebuild backend: `docker compose build backend`
2. Reiniciar: `docker compose up -d`
3. Testar fluxo completo:
   - Fazer `initialize` e capturar `Mcp-Session-Id` da resposta
   - Fazer `tools/list` passando o `Mcp-Session-Id` no header

**Comandos de teste (PowerShell):**
```powershell
# Initialize
$body = '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"mcpx-test","version":"1.0"}},"id":1}'
$response = Invoke-WebRequest -Uri "http://localhost:8080/mcp/c4de33bf-9875-48e0-97ce-43f8364c6e53/DeepWiki" -Method POST -Body $body -ContentType "application/json" -Headers @{"Accept"="application/json, text/event-stream"}
$sessionId = $response.Headers["Mcp-Session-Id"]
Write-Host "Session ID: $sessionId"

# Tools list (com session ID)
$toolsBody = '{"jsonrpc":"2.0","method":"tools/list","id":2}'
Invoke-RestMethod -Uri "http://localhost:8080/mcp/c4de33bf-9875-48e0-97ce-43f8364c6e53/DeepWiki" -Method POST -Body $toolsBody -ContentType "application/json" -Headers @{"Accept"="application/json, text/event-stream"; "Mcp-Session-Id"="$sessionId"}
```

---

## ✅ IMPLEMENTADO E FUNCIONANDO

### Health Check System
- Background job (5min interval) em `services/health_check.rs`
- Status machine: healthy/unhealthy/pending_auth/pending_health
- Countdown timer no frontend
- `last_health_check` salvo no teste inicial

### OAuth 2.1 Auto-discovery
- RFC 8414 (metadata discovery)
- RFC 7591 (dynamic client registration)
- PKCE flow completo
- Tokens encriptados (AES-256-GCM)
- SDK OAuth no frontend (`lib/mcp/`)

### Server Setup UX
- `ServerSetupModal.vue` com 4 steps
- OAuth polling até conexão
- Opção "Fazer Depois" para skip OAuth
- Mensagens de erro limpas

### Frontend Improvements
- Health Check card com countdown
- Token expiration detection
- Re-auth flow com polling

---

## 📋 A FAZER (PRÓXIMAS PRIORIDADES)

### 1. Validar Proxy MCP (URGENTE)
- [ ] Testar fluxo completo initialize → tools/list
- [ ] Testar com cloudfare e DeepWiki
- [ ] Verificar SSE streaming funciona

### 2. Testar Outros Tipos de Auth
Servidores cadastrados para teste:
- `cloudfare` - https://docs.mcp.cloudflare.com/mcp (Auth: None)
- `DeepWiki` - https://mcp.deepwiki.com/mcp (Auth: None)

Matriz de testes em: `auth_test_plan.md` (nos artifacts)

### 3. Features Pendentes (Implementation Plan)
- [ ] Auto-refresh OAuth tokens (`try_refresh_token` é TODO)
- [ ] Tool governance (whitelist/blacklist)
- [ ] Virtual gateways
- [ ] Audit logging

---

## 🗂️ ESTRUTURA DE ARQUIVOS CHAVE

```
backend/
├── src/
│   ├── main.rs              # Rotas definidas aqui
│   ├── routes/
│   │   ├── proxy.rs         # ⚠️ MCP Proxy (não validado)
│   │   ├── servers.rs       # CRUD servers
│   │   ├── oauth.rs         # OAuth endpoints
│   │   └── auth.rs          # User auth
│   └── services/
│       ├── health_check.rs  # Background job
│       └── crypto.rs        # Encryption

frontend/
├── src/
│   ├── components/ui/
│   │   └── ServerSetupModal.vue  # Setup wizard
│   ├── lib/mcp/                  # OAuth SDK
│   └── views/dashboard/
│       ├── ServerDetails.vue     # Health check card
│       └── ServerNew.vue         # Usa ServerSetupModal
```

---

## 🔧 COMANDOS ÚTEIS

```bash
# Rebuild backend
docker compose build backend

# Reiniciar tudo
docker compose up -d

# Ver logs backend
docker logs -f mcpx-backend

# Acessar
# Frontend: http://localhost:3000
# Backend API: http://localhost:8080/api
# MCP Proxy: http://localhost:8080/mcp/{user_id}/{server_name}
```

---

## 📚 REFERÊNCIAS

- **MCP Spec Transports:** https://modelcontextprotocol.io/specification/2025-11-25/basic/transports
- **Session Management:** Mcp-Session-Id header obrigatório após initialize
- **Accept header:** Deve ser `application/json, text/event-stream`
