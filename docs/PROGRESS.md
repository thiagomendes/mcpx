# MCPX - Estado do Projeto e Próximos Passos

**Última atualização:** 2025-12-22 19:30
**Branch:** feature/request-metrics

---

## 🚧 EM DESENVOLVIMENTO

### Observability Platform (TimescaleDB)

**Status:** 🔄 **EM ANDAMENTO**

**Estratégia:** TimescaleDB (extensão PostgreSQL para time-series)

#### Phase 1: Esta Entrega

| Feature | Escopo | Status |
|---------|--------|--------|
| **Request Metrics** | Completo (migration + service + API + dashboard) | 🔄 |
| **Server Logs** | Schema only (hypertable) | ⏳ |
| **Audit Trail** | Schema only (hypertable) | ⏳ |

**Checklist:**
- [/] Migration com hypertables e continuous aggregates
- [ ] Serviço de métricas (`services/metrics.rs`)
- [ ] Integração no proxy.rs (timing + record)
- [ ] API routes (`/api/metrics/today`, `/api/metrics/hourly`)
- [ ] Dashboard com contadores reais
- [ ] Testes unitários

#### Phase 2: Próxima Entrega

| Feature | Descrição |
|---------|-----------|
| **Analytics Dashboard** | Gráficos, top tools, tendências |
| **Rate Limiting** | Contador por janela, middleware |
| **Alerting** | Picos de erro, notificações |

**Benefícios da abordagem:**
- Particionamento automático por tempo
- Retenção configurável (90 dias raw, 1 ano aggregates)
- Continuous aggregates para performance
- Mesmo PostgreSQL, sem nova infraestrutura

---

## ✅ IMPLEMENTADO E FUNCIONANDO

### Virtual Gateways

**Status:** ✅ **COMPLETO**

- Agregar múltiplos servidores em um endpoint
- Gateway CRUD (create, read, update, delete)
- Proxy routing com governance prefix
- UI: lista, detalhes, gerenciamento de servidores
- Dashboard com seção dedicada
- 8 testes unitários (frontend)

---

### Tool Governance (`/servers/:name/governance`)

**Status:** ✅ **VALIDADO** - 8 casos de teste passaram!

- Allowlist/Blocklist de tools
- Prefix global (ex: `cf_search_docs`)
- Validação no tools/call
- UI completa com chips clicáveis

---

### Code Quality Refactoring

**Status:** ✅ **COMPLETO**

- Backend: mensagens centralizadas, SQL constants, OAuth abstraction
- Frontend: ESLint configurado, constants centralizados
- Clippy warnings: 10 | ESLint warnings: 0
- **37 testes passando** (20 backend + 17 frontend)

---

### MCP Proxy Layer

**Status:** ✅ **VALIDADO**

- Multi-tenant isolation (`/mcp/:user_id/:server_name`)
- Auth injection (API Key, Bearer, OAuth)
- Governance filter integrado

---

### OAuth 2.1 Auto-discovery
- RFC 8414 + RFC 7591 + PKCE
- Tokens AES-256-GCM encriptados

### Health Check System
- Background job (5min interval)
- Status machine + countdown timer

---

## 📋 A FAZER (FUTURO)

### Multi-provider Auth
- [ ] Microsoft Entra ID login
- [ ] GitHub login
- [ ] Abstração já criada

### API Key Protection
- [ ] Proteger server com API key
- [ ] Dashboard para gerar/revogar

### Auto-refresh OAuth tokens
- [ ] Implementar `try_refresh_token`
- [ ] Usar refresh_token quando expira

---

## 🔧 COMANDOS ÚTEIS

```bash
# Rebuild
docker compose build && docker compose up -d

# Lint
cd backend && cargo clippy
cd frontend && npm run lint

# Testes
cd backend && cargo test
cd frontend && npm test
```

---

## 📚 REFERÊNCIAS

- **MCP Spec:** https://modelcontextprotocol.io/specification/2025-11-25
- **Governance Tests:** `docs/GOVERNANCE_TEST_SCRIPT.md`
