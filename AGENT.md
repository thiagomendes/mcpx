# MCPX Agent Guidelines

Coding best practices and conventions for this project.

---

## 📁 Project Structure

```
backend/                  # Rust + Axum
├── src/
│   ├── main.rs          # Routes and server setup
│   ├── config.rs        # Environment configuration
│   ├── messages.rs      # Centralized error messages
│   ├── routes/          # HTTP handlers
│   │   ├── proxy.rs     # MCP Proxy with governance
│   │   ├── metrics.rs   # Metrics API endpoints
│   │   └── ...
│   ├── services/        # Business logic
│   │   ├── metrics.rs   # TimescaleDB metrics service
│   │   └── ...
│   └── models/          # Structs and types
├── migrations/          # SQL migrations (sqlx + TimescaleDB)
└── Cargo.toml

frontend/                 # Vue 3 + TypeScript + Vite
├── src/
│   ├── constants.ts     # Centralized constants
│   ├── api/             # HTTP client (axios)
│   ├── components/      # Vue components
│   ├── lib/             # Utilities and abstractions
│   ├── stores/          # Pinia stores
│   └── views/           # Pages
│       └── dashboard/
│           └── Dashboard.vue  # Metrics dashboard
├── eslint.config.js
└── package.json

mcp-servers-examples/     # Example MCP servers for testing
├── weather/              # API Key auth
├── utilities-mcp/        # Bearer Token auth
└── filesystem-mcp/       # OAuth Client Credentials

k8s/                      # Kubernetes manifests
├── base/                 # Base resources
└── overlays/             # Environment overlays (local, production)
```

---

## ✅ DO

### General
- Use constants for repeated strings (error messages, SQL queries)
- Prefix unused variables with `_` (e.g., `_e`, `_wrapper`)
- Keep unit tests passing before committing
- Use descriptive commits: `feat:`, `fix:`, `chore:`, `refactor:`
- **Always create unit tests** for new features before delivery

### Backend (Rust)
- Error messages in `messages.rs`
- SQL queries as `const` at top of route files
- Use `#[allow(dead_code)]` for preparatory code
- Run `cargo clippy` before push
- Run `cargo test` to validate

### Frontend (Vue + TypeScript)
- Constants in `constants.ts` (ROUTES, API_ENDPOINTS, ERROR_MESSAGES)
- Provider abstractions in `lib/` (identityProviders.ts, etc)
- Use `unknown` instead of `any` in catch blocks
- Run `npm run lint` before push
- Run `npm test` to validate

---

## ❌ DON'T
### Code Quality
- Avoid excessive comments - code should be self-documenting
- Don't add comments that just repeat what the code does
- Write clear, readable code that explains itself


### General
- Don't leave `.log` or temp files in root
- Don't commit with failing tests
- Don't hardcode error messages

### Backend
- Don't use inline SQL - create constants
- Don't ignore clippy warnings without explicit `#[allow(...)]`
- Don't mix business logic in routes (use services/)

### Frontend
- Don't use `any` type - use `unknown` or specific types
- Don't duplicate constants across files
- Don't leave unused variables without `_` prefix

---

## 🔍 Where to Find

| What | Location |
|------|----------|
| Error messages (backend) | `backend/src/messages.rs` |
| Constants (frontend) | `frontend/src/constants.ts` |
| MCP Proxy routes | `backend/src/routes/proxy.rs` |
| Metrics API | `backend/src/routes/metrics.rs` |
| Metrics Service | `backend/src/services/metrics.rs` |
| Dashboard UI | `frontend/src/views/dashboard/Dashboard.vue` |
| Governance API | `backend/src/routes/governance.rs` |
| Governance UI | `frontend/src/components/ui/ToolGovernance.vue` |
| OAuth providers (backend) | `backend/src/services/oauth_provider.rs` |
| Identity providers (frontend) | `frontend/src/lib/identityProviders.ts` |
| Governance tests | `frontend/src/components/__tests__/ToolGovernance.spec.ts` |
| SQL migrations | `backend/migrations/` |
| Load test script | `scripts/load-test.sh` |

---

## 🧪 Validation Commands

```bash
# Backend
cd backend
cargo clippy        # Lint
cargo test          # Unit tests
cargo build         # Compile

# Frontend
cd frontend
npm run lint        # ESLint
npm test            # Vitest
npm run build       # Production build

# Docker
docker compose build && docker compose up -d
docker logs -f mcpx-backend
```

---

## 🔧 Lint Configuration

### ESLint (frontend)
- `varsIgnorePattern: '^_'` - ignores variables with `_` prefix
- `argsIgnorePattern: '^_'` - ignores arguments with `_` prefix
- `no-explicit-any: warn` - warns on `any` usage

### Clippy (backend)
- `dead_code` warnings allowed for preparatory code
- Use `#[allow(dead_code)]` when necessary

---

## 📝 Code Patterns

### Rust - Error Handling
```rust
use crate::messages::error;

.map_err(|e| (StatusCode::NOT_FOUND, format!("{}: {}", error::SERVER_NOT_FOUND, e)))?
```

### TypeScript - Catch Blocks
```typescript
} catch (e: unknown) {
  const err = e as { response?: { data?: string } }
  error.value = err.response?.data || 'Default error'
}
```

### Vue - Unused Variables
```typescript
// Correct - underscore prefix
} catch (_e) {
  // handle silently
}
```

---

## 🔐 Security

- OAuth tokens encrypted with AES-256-GCM
- Encryption key in `ENCRYPTION_KEY` env var
- JWT secret in `JWT_SECRET` env var
- Never log tokens or secrets
