# MCPX - Estado do Projeto e Próximos Passos

**Última atualização:** 2025-12-21 09:50
**Commit mais recente:** 2042d2d (fix: MCP Proxy validated - single route for multi-tenant isolation)

---

## ✅ IMPLEMENTADO E FUNCIONANDO

### MCP Proxy Layer (`/mcp/:user_id/:server_name`)

**Arquivo:** `backend/src/routes/proxy.rs`

**Status:** ✅ **VALIDADO** - Funcionando corretamente!

**O que foi feito:**
- Handler `mcp_proxy` em `proxy.rs`
- Rota única `/mcp/:user_id/:server_name` (user_id para isolamento multi-tenant)
- Rota pública por enquanto (API key opcional no futuro)
- `Accept: application/json, text/event-stream` header
- Forwarding do header `Mcp-Session-Id`
- Injeção de auth headers (API Key, Bearer, OAuth)

**Testes realizados (2025-12-21):**
- ✅ `initialize` funcionou (DeepWiki e Cloudflare)
- ✅ `tools/list` funcionou com `Mcp-Session-Id`
- ✅ Latência do proxy: ~100-200ms overhead (aceitável)

**Comandos de teste (bash):**
```bash
# 1. Initialize e captura Session ID
SESSION_ID=$(curl -s -i -X POST http://localhost:8080/mcp/{user_id}/{server_name} \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"mcpx-test","version":"1.0"}},"id":1}' \
  | grep "^mcp-session-id:" | cut -d' ' -f2 | tr -d '\r\n')

echo "Session ID: $SESSION_ID"

# 2. tools/list
curl -s -X POST http://localhost:8080/mcp/{user_id}/{server_name} \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":2}'
```

---

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

### 1. Auto-refresh OAuth tokens
- [ ] Implementar `try_refresh_token` em `health_check.rs` (atualmente é TODO)
- [ ] Usar refresh_token para obter novo access_token quando expirado

### 2. Tool Governance (whitelist/blacklist)
- [ ] UI para configurar whitelist/blacklist de tools
- [ ] Filtrar tools no proxy antes de retornar ao cliente
- [ ] Schema já existe: `governance_configs`

### 3. Virtual Gateways
- [ ] Agregar múltiplos servidores em um único endpoint
- [ ] Ex: `/mcp/{user_id}/all-tools` combina GitHub + Linear + Slack

### 4. Audit Logging
- [ ] Registrar requests MCP para compliance/analytics
- [ ] Schema já existe: `audit_logs`

### 5. API Key Protection (opcional por server)
- [ ] Usuário pode proteger server exposto com API key
- [ ] Gerar/revogar API keys via dashboard

### 6. SSE Streaming real
- [ ] Atualmente lê toda resposta e devolve
- [ ] Para respostas grandes, fazer streaming progressivo

---

## 🗂️ ESTRUTURA DE ARQUIVOS CHAVE

```
backend/
├── src/
│   ├── main.rs              # Rotas definidas aqui
│   ├── routes/
│   │   ├── proxy.rs         # ✅ MCP Proxy (validado)
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
