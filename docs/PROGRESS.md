# MCPX - Estado do Projeto

**Última atualização:** 2025-12-23
**Branch:** feature/request-metrics

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

### Code Quality

**Status:** ✅ **COMPLETO**

- Backend: mensagens centralizadas, SQL constants
- Frontend: ESLint configurado, constants centralizados
- **42 testes passando** (20 backend + 22 frontend)
- Clippy warnings: 12 (complexity/dead_code)
- ESLint warnings: 2 (any types em charts)

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

## 🚧 EM DESENVOLVIMENTO

#### Alerting System (Alert Configuration Builder)
**Prioridade:** 🟡 Média

Builder visual de regras de alerta inspirado no Datadog, simplificado para MCPX.

##### UI Design - Estrutura em 3 Passos

```
┌─────────────────────────────────────────────────────────────────┐
│  Create Alert Rule                                              │
├─────────────────────────────────────────────────────────────────┤
│  ① Choose Alert Type                                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │ 📊 Threshold │ │ 📈 Spike     │ │ ❌ No Data   │            │
│  └──────────────┘ └──────────────┘ └──────────────┘            │
│                                                                 │
│  ② Define the Condition                                        │
│  When [Error Rate ▼] for [deepwiki ▼]                          │
│  is [above ▼] [5] [%] for [5 ▼] minutes                        │
│                                                                 │
│  ③ Notification                                                │
│  Alert Name: [High error rate on deepwiki]                     │
│  [✓] Dashboard notification                                    │
│  [ ] Email (coming soon)                                       │
└─────────────────────────────────────────────────────────────────┘
```

##### Alert Types

| Tipo | Descrição |
|------|-----------|
| **Threshold** | Métrica cruza um valor (error rate > 5%) |
| **Spike** | Mudança brusca (latency +200% em 5min) |
| **No Data** | Servidor sem resposta (sem requests há 10min) |

##### Metrics

| Métrica | Unidade |
|---------|---------|
| Error Rate | % |
| Avg Latency | ms |
| Request Count | requests |
| P95 Latency | ms |

##### Database Schema

```sql
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    alert_type VARCHAR(20) NOT NULL,  -- 'threshold', 'spike', 'no_data'
    metric VARCHAR(50) NOT NULL,       -- 'error_rate', 'avg_latency', etc
    scope_type VARCHAR(20) NOT NULL,   -- 'all', 'server', 'gateway'
    scope_id UUID,
    operator VARCHAR(10) NOT NULL,     -- 'above', 'below'
    threshold FLOAT NOT NULL,
    duration_minutes INT NOT NULL,
    notify_dashboard BOOLEAN DEFAULT true,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE alert_history (
    id UUID PRIMARY KEY,
    rule_id UUID REFERENCES alert_rules(id),
    triggered_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    trigger_value FLOAT NOT NULL,
    status VARCHAR(20) NOT NULL  -- 'triggered', 'resolved', 'acknowledged'
);
```

##### API Endpoints

| Method | Endpoint | Descrição |
|--------|----------|-----------|
| GET | `/api/alerts` | Listar regras |
| POST | `/api/alerts` | Criar regra |
| PUT | `/api/alerts/{id}` | Atualizar regra |
| DELETE | `/api/alerts/{id}` | Deletar regra |
| GET | `/api/alerts/history` | Histórico |
| POST | `/api/alerts/{id}/acknowledge` | Marcar como visto |

##### Tarefas

- [ ] Migração: Tabela `alert_rules`
- [ ] Migração: Tabela `alert_history`
- [ ] Backend: Job de verificação (cron 1min)
- [ ] API: CRUD de regras
- [ ] Frontend: `AlertBuilder.vue` (modal 3 steps)
- [ ] Frontend: `AlertsList.vue` (página de regras)
- [ ] Frontend: Painel de alertas ativos no dashboard
- [ ] (Futuro) Email/Slack/webhook

---

## 📋 A FAZER (FUTURO)

### Multi-provider Auth
- [ ] Microsoft Entra ID login
- [ ] GitHub login

### API Key Protection
- [ ] Proteger server com API key
- [ ] Dashboard para gerar/revogar

### Auto-refresh OAuth tokens
- [ ] Implementar `try_refresh_token`
- [ ] Usar refresh_token quando expira

### Rate Limiting
- [ ] Contador por janela de tempo
- [ ] Middleware de limitação
- [ ] (Considerar PostgreSQL em vez de Redis)

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
| Auth | Google OAuth 2.1 + JWT |
| Containerização | Docker Compose |

---

## 📚 REFERÊNCIAS

- **MCP Spec:** https://modelcontextprotocol.io/specification/2025-11-25
- **Governance Tests:** `docs/GOVERNANCE_TEST_SCRIPT.md`
- **Test Servers:** `docs/TEST_SERVERS.md`
