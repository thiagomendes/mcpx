# MCPX - Infrastructure Guide

**Purpose:** Setup guide for local development and deployment.
**Stack:** Docker Compose

---

## 📖 Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Services](#services)
- [Environment Variables](#environment-variables)
- [Development Workflow](#development-workflow)
- [Database](#database)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

```bash
# Docker & Docker Compose
docker --version    # 24.x+
docker compose version  # 2.x+

# Node.js (for local frontend dev)
node --version      # v20.x+

# Rust (for local backend dev)
rustc --version     # 1.75+
```

---

## Quick Start

### 1. Clone and Setup

```bash
git clone https://github.com/thiagomendes/mcpx.git
cd mcpx

# Copy environment template (if exists)
cp .env.example .env  # Edit with your Google OAuth credentials
```

### 2. Start All Services

```bash
docker compose up -d
```

### 3. Access Application

| Service | URL |
|---------|-----|
| Frontend (Dashboard) | http://localhost:3000 |
| Backend API | http://localhost:8080 |
| Database | localhost:5432 |

### 4. Verify Services

```bash
# Check all containers
docker compose ps

# Expected output:
# mcpx-postgres   Up (healthy)
# mcpx-backend    Up
# mcpx-frontend   Up

# Check backend health
curl http://localhost:8080/api/health
# Output: OK
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Docker Compose                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────┐ │
│  │   Frontend   │───▶│   Backend    │───▶│  TimescaleDB  │ │
│  │  (Vue 3)     │    │  (Rust/Axum) │    │  (PostgreSQL) │ │
│  │  :3000       │    │  :8080       │    │  :5432        │ │
│  └──────────────┘    └──────────────┘    └───────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Services

### TimescaleDB (PostgreSQL 15)

Time-series database for metrics and application data.

```yaml
image: timescale/timescaledb:latest-pg15
ports: 5432:5432
```

**Features used:**
- Hypertables for `request_metrics` (auto-partitioned by time)
- Continuous aggregates for hourly stats
- Retention policies (90 days raw data)

### Backend (Rust + Axum)

API server and MCP proxy.

```yaml
build: ./backend
ports: 8080:8080
```

**Key routes:**
- `/api/*` - REST API
- `/mcp/:user_id/:server` - MCP Proxy

### Frontend (Vue 3 + Vite)

Web dashboard with hot-reload.

```yaml
build: ./frontend
ports: 3000:3000
volumes:
  - ./frontend:/app  # Hot reload
```

---

## Environment Variables

### Required for Production

| Variable | Description | Example |
|----------|-------------|---------|
| `GOOGLE_CLIENT_ID` | Google OAuth Client ID | `xxxx.apps.googleusercontent.com` |
| `GOOGLE_CLIENT_SECRET` | Google OAuth Secret | `GOCSPX-xxxx` |
| `JWT_SECRET` | Secret for JWT signing | `openssl rand -base64 32` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `DEV_MODE` | `false` | Skip OAuth in development |
| `DEV_USER_ID` | - | User ID for DEV_MODE |
| `RUST_LOG` | `info` | Log level |

### Example `.env`

```bash
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=GOCSPX-your-secret
JWT_SECRET=your-random-32-char-secret

# For local development without OAuth
DEV_MODE=true
DEV_USER_ID=your-user-uuid
```

---

## Development Workflow

### Full Stack (Docker)

```bash
# Start all services
docker compose up -d

# Watch logs
docker compose logs -f backend
docker compose logs -f frontend

# Rebuild after code changes
docker compose build backend && docker compose up -d backend
docker compose build frontend && docker compose up -d frontend
```

### Backend Only (Local Rust)

```bash
# Start only DB
docker compose up -d postgres

# Run backend locally
cd backend
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/mcpx
cargo run

# Run tests
cargo test
cargo clippy
```

### Frontend Only (Local Node)

```bash
# Start backend in Docker
docker compose up -d postgres backend

# Run frontend locally (faster hot reload)
cd frontend
npm install
npm run dev

# Run tests
npm test
npm run lint
```

---

## Database

### Connect to Database

```bash
# Via Docker
docker exec -it mcpx-postgres psql -U postgres -d mcpx

# Or with any PostgreSQL client
psql postgresql://postgres:postgres@localhost:5432/mcpx
```

### Run Migrations

Migrations run automatically on backend startup via SQLx.

```bash
# Manual migration (if needed)
cd backend
sqlx migrate run
```

### View TimescaleDB Stats

```sql
-- Check hypertable info
SELECT * FROM timescaledb_information.hypertables;

-- View continuous aggregates
SELECT * FROM timescaledb_information.continuous_aggregates;

-- Recent metrics
SELECT * FROM request_metrics ORDER BY time DESC LIMIT 10;

-- Hourly aggregate
SELECT * FROM request_metrics_hourly ORDER BY bucket DESC LIMIT 10;
```

---

## Troubleshooting

### Container won't start

```bash
# Check logs
docker compose logs backend

# Common fixes
docker compose down
docker compose build --no-cache
docker compose up -d
```

### Database connection error

```bash
# Verify postgres is healthy
docker compose ps

# Check if migrations ran
docker compose logs backend | grep -i migration
```

### Frontend can't reach backend

```bash
# Verify backend is running
curl http://localhost:8080/api/health

# Check frontend logs
docker compose logs frontend
```

### OAuth not working

1. Verify Google OAuth credentials in `.env`
2. Check redirect URI in Google Console matches: `http://localhost:8080/api/auth/google/callback`
3. Use `DEV_MODE=true` for local testing without OAuth

---

## Google OAuth Setup

### 1. Create Project

1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Create new project: "mcpx"
3. Enable "Google+ API"

### 2. Create OAuth Credentials

1. Go to APIs & Services → Credentials
2. Create OAuth 2.0 Client ID
3. Application type: Web application
4. Authorized JavaScript origins: `http://localhost:3000`
5. Authorized redirect URI: `http://localhost:8080/api/auth/google/callback`

### 3. Configure Environment

```bash
export GOOGLE_CLIENT_ID=your-client-id
export GOOGLE_CLIENT_SECRET=your-secret
docker compose up -d
```

---

## Useful Commands

```bash
# Start
docker compose up -d

# Stop
docker compose down

# Rebuild everything
docker compose build && docker compose up -d

# View logs
docker compose logs -f

# Clean up (including volumes)
docker compose down -v

# Check resource usage
docker stats
```

---

**Questions?** mail.thiagomendes@gmail.com
