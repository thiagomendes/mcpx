# MCPX - Estado do Projeto

**Última atualização:** 2026-01-04
**Branch:** main

---

## ✅ IMPLEMENTADO E FUNCIONANDO

### Distributed Workers & Gateway Session Fix

**Status:** ✅ **COMPLETO**

Arquitetura distribuída para background jobs e correção crítica do gateway.

| Feature | Descrição | Status |
|---------|-----------|--------|
| Web/Worker separation | Binários separados (`mcpx-web`, `mcpx-worker`) | ✅ |
| PostgreSQL Job Queue | Locking distribuído com `FOR UPDATE SKIP LOCKED` | ✅ |
| Centralized Server Auth | `server_auth.rs` com suporte a todos auth types | ✅ |
| Gateway Session Fix | Preserva `gw_*` session ID nas respostas | ✅ |
| OAuth Token Persistence | `oauth_client_credentials` persiste tokens no DB | ✅ |

**Testado:** 17/17 tools gateway funcionando (100%)

---

## ✅ IMPLEMENTADO E FUNCIONANDO

### Observability Platform (TimescaleDB)

**Status:** ✅ **COMPLETO**

**Infraestrutura:**
- TimescaleDB (PostgreSQL 15 com extensão time-series)
- Hypertables para `request_metrics` com particionamento automático
- Continuous aggregates para métricas horárias
- Retenção configurável (90 dias raw, 1 ano aggregates)

**Features implementadas:**

| Feature | Descrição | Status |
|---------|-----------|--------|
| Request Metrics | Coleta de latência, success rate, tool_name | ✅ |
| Tool Name Tracking | Extração de tool name de `tools/call` requests | ✅ |
| Flexible Query API | `/api/metrics/query` com group_by, filters, time_bucket | ✅ |
| Dashboard Charts | Server Requests, Server Latency, Tool Calls, Tool Latency | ✅ |
| Time Range Selection | 15min, 1h, 6h, 24h, 7d | ✅ |
| Server Filter | Filtrar métricas por servidor específico | ✅ |
| Load Test Script | `scripts/load-test.sh` com profiles low/medium/high | ✅ |

---

### Virtual Gateways

**Status:** ✅ **COMPLETO**

- Agregar múltiplos servidores em um endpoint
- Gateway CRUD (create, read, update, delete)
- Proxy routing com governance prefix
- UI: lista, detalhes, gerenciamento de servidores
- 8 testes unitários (frontend)

---

### Tool Governance

**Status:** ✅ **COMPLETO**

- Allowlist/Blocklist de tools
- Prefix global (ex: `cf_search_docs`)
- Validação no `tools/call`
- UI completa com chips clicáveis
- 9 testes unitários (frontend)

---

### MCP Proxy Layer

**Status:** ✅ **COMPLETO**

- Multi-tenant isolation (`/mcp/:user_id/:server_name`)
- Auth injection (API Key, Bearer, OAuth)
- Governance filter integrado
- Tool name extraction para métricas

---

### OAuth 2.1 Auto-discovery

**Status:** ✅ **COMPLETO**

- RFC 8414 + RFC 7591 + PKCE
- Tokens AES-256-GCM encriptados
- Suporte a Google OAuth

---

### Health Check System

**Status:** ✅ **COMPLETO**

- Background job (5min interval)
- Status machine + countdown timer

---

### MCP Server Examples

**Status:** ✅ **COMPLETO**

3 servidores de exemplo para testar diferentes métodos de autenticação:

| Server | Auth Type | Location |
|--------|-----------|----------|
| weather-mcp | API Key | `mcp-servers-examples/weather/` |
| utilities-mcp | Bearer Token | `mcp-servers-examples/utilities-mcp/` |
| filesystem-mcp | OAuth Client Credentials | `mcp-servers-examples/filesystem-mcp/` |

**Features:**
- Docker integrado no `docker-compose.yml` principal
- Test script completo: `mcp-servers-examples/TEST_SCRIPT.md`
- Health check authenticado para todos os tipos
- Gateway proxy com auth injection completo

---

### Code Quality

**Status:** ✅ **COMPLETO**

- Backend: mensagens centralizadas, SQL constants
- Frontend: ESLint configurado, constants centralizados
- **102 testes passando** (backend Rust)
- Clippy warnings: 0
- ESLint warnings: 0

---

## ✅ IMPLEMENTADO E FUNCIONANDO

### Observability Phase 2

**Status:** ✅ **COMPLETO**

O dashboard de métricas foi expandido com features completas de observability:

#### Audit Log Viewer ✅
**Prioridade:** 🔴 Alta - **IMPLEMENTADO**

Página para visualizar logs individuais de cada request MCP:

| Campo | Descrição | Status |
|-------|-----------|--------|
| Timestamp | Data/hora do request | ✅ |
| Server/Gateway | Nome do target | ✅ |
| Method | `initialize`, `tools/list`, `tools/call` | ✅ |
| Tool Name | Nome da tool (se `tools/call`) | ✅ |
| Request Body | JSON completo do request | ✅ |
| Response Body | JSON completo da resposta (SSE parsed) | ✅ |
| Latency | Tempo de resposta (ms) | ✅ |
| Status | ✅ Success / ❌ Error (JSON-RPC aware) | ✅ |

**Features implementadas:**
- [x] Migração: `audit_logs` hypertable com 30 dias retention
- [x] API: `GET /api/audit` com paginação e filtros
- [x] API: `GET /api/audit/{id}` para detalhes
- [x] Frontend: Página `/audit` com tabela paginada
- [x] Frontend: Modal de detalhes do request
- [x] Frontend: Filtros (server, method, date range, status)
- [x] Frontend: Auto-refresh com toggle e countdown
- [x] SSE parsing para capturar response bodies de MCP servers
- [x] Detecção de erros JSON-RPC no response body

---

#### Percentile Metrics ✅
**Prioridade:** 🟢 Baixa - **IMPLEMENTADO**

- [x] API: `GET /api/metrics/summary` com P50, P95, P99, MAX
- [x] Frontend: Toggle buttons no dashboard para latency/throughput
- [x] Suporte a MIN/MAX para throughput

---

#### Server Logs ✅
**Prioridade:** 🟡 Média - **COBERTO PELO AUDIT LOG**

O Audit Log Viewer implementado cobre completamente o escopo original de Server Logs:
- Logs por servidor (filtro por `target_name`)
- Visualização de erros com stack trace
- Request/Response body completos

---


#### Alerting System ✅
**Prioridade:** 🟡 Média - **COMPLETO**

Sistema de alertas com UI visual, background job, e painel de alertas ativos no Dashboard.

##### Implementado

**Backend:**
- [x] Migração: Tabela `alert_rules` com suporte a threshold/spike/no_data
- [x] Migração: Tabela `alert_history` para tracking de alertas
- [x] Service: `services/alerts.rs` com CRUD completo
- [x] Background Job: `jobs/alert_evaluator.rs` (intervalo 60s, configurável)
- [x] APIs: CRUD de regras, alertas ativos, histórico, acknowledge

**Frontend:**
- [x] Store: `stores/alerts.ts` com state management
- [x] Page: `views/alerts/AlertsList.vue` com tabela de regras
- [x] Component: `AlertBuilder.vue` (wizard 2 passos simplificado)
- [x] Component: `ActiveAlertsPanel.vue` no Dashboard
- [x] Dropdown de seleção de server/gateway específico
- [x] Badge com nome do server no alerta ativo

**Design para Extração:**
- [x] `jobs/alert_evaluator.rs` é stateless
- [x] Comunica apenas via banco de dados
- [x] `ALERT_EVAL_INTERVAL_SECONDS` configurável

##### Dashboard Melhorias (bônus)
- [x] Linhas separadas por server nos gráficos (All Servers mode)
- [x] Layout full-width para gráficos de Server
- [x] Toggle de percentil (avg/p50/p95/p99/max) no gráfico de Latency
- [x] Toggle all/success/errors no gráfico de Requests
- [x] Backend retorna percentis (p50/p95/p99/max) por time bucket

---

### Organizations & Multi-Provider Auth

**Status:** ✅ **COMPLETO**

Refactor completo para arquitetura multi-tenant baseada em Organizações.

- **Multi-Tenant:** Recursos (servers, logs, metrics) isolados por `org_id`
- **Auth Providers:** Google, GitHub, Microsoft Entra ID (com conta unificada por email)
- **Member Roles:** Owner, Admin, Member
- **Invite Flow:** Links de convite seguros e UI de aceitação explícita
- **Org Management:** Criação, troca e remoção de organizações
- **Spec:** [Authentication & Authorization](./authentication_spec.md) - Arquitetura de segurança (PATs, RBAC)

---

## 📋 A FAZER (FUTURO)

### Gateway Authentication (PATs & Service Accounts) ✅
**Status:** ✅ **COMPLETO**

Implementação completa de autenticação para consumo de servidores expostos:

#### Personal Access Tokens (PATs) ✅
- [x] Criar tabela `personal_access_tokens`
- [x] Implementar API CRUD para PATs
- [x] Validar PATs no MCP Proxy
- [x] UI para gerar/revogar tokens
- [x] PATs self-service para members

#### Service Accounts (M2M) ✅
- [x] Criar tabela `service_accounts` com coluna `role`
- [x] Implementar OAuth Client Credentials flow
- [x] UI para gerenciar service accounts com roles (Admin/Member)

---

### Multi-Token Dashboard API ✅
**Status:** ✅ **COMPLETO**

Dashboard API agora aceita 3 tipos de token:

| Token Type | Exemplo | Uso |
|------------|---------|-----|
| PAT | `mcpx_pat_xxx` | Users programmatic access |
| M2M JWT | `eyJhb...` (token_type=m2m) | Service accounts |
| Web Session | `eyJhb...` | Browser login |

**RBAC testado:**
- 102 testes unitários
- 90 testes de integração (5 token types × 18 endpoints)
- Permissões: Member=read, Admin/Owner=read+write

**Security fixes:**
- [x] AuthUser rejeita requests de usuários removidos da org
- [x] Validação de email no aceite de convite
- [x] PATs deletados ao remover membro da org

### Auto-refresh OAuth tokens ✅
**Status:** ✅ **COMPLETO**

- [x] `try_refresh_token` implementado em `services/health_check.rs`
- [x] Usa refresh_token quando access_token expira
- [x] Atualiza banco com novo token automaticamente

### Rate Limiting
- [ ] Contador por janela de tempo
- [ ] Middleware de limitação
- [ ] (Considerar PostgreSQL em vez de Redis)

### Test Coverage Improvement
- [ ] Atingir 90% de cobertura
- [ ] Configurar `cargo-tarpaulin` (Rust)
- [ ] Configurar `@vitest/coverage-v8` (Frontend)

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

# Load Test
./scripts/load-test.sh low    # 100 requests/tool
./scripts/load-test.sh medium # 500 requests/tool
./scripts/load-test.sh high   # 1000 requests/tool
```

---

## 🏗️ STACK

| Componente | Tecnologia |
|------------|------------|
| Backend | Rust + Axum |
| Frontend | Vue 3 + TypeScript + Vite |
| Database | TimescaleDB (PostgreSQL 15) |
| Auth | Google, GitHub, Microsoft OAuth 2.1 + JWT |
| Containerização | Docker Compose |

---

## 📚 REFERÊNCIAS

- **MCP Spec:** https://modelcontextprotocol.io/specification/2025-11-25
- **Governance Tests:** `docs/GOVERNANCE_TEST_SCRIPT.md`
- **Test Servers:** `docs/TEST_SERVERS.md`
