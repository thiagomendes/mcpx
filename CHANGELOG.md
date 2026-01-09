# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] Incubating Release

### Core Platform

- MCP Proxy with multi-tenant isolation (`/mcp/:org_slug/:server`)
- Virtual Gateways for aggregating multiple servers into single endpoint
- Tool Governance with allowlist/blocklist and prefix configuration
- Distributed worker architecture with PostgreSQL job queue

### Authentication

- OAuth with Google, GitHub, and Microsoft providers
- Personal Access Tokens (PATs) for programmatic access
- Service Accounts with OAuth Client Credentials flow
- Multi-provider account linking by email

### Observability

- TimescaleDB integration for time-series metrics
- Real-time dashboard with latency and throughput charts
- Percentile metrics (P50, P95, P99, MAX)
- Audit log viewer with request/response inspection
- Admin action logging (22 audited operations)

### Alerting

- Configurable alert rules (threshold, spike, no-data)
- Background evaluation with 60-second intervals
- Alert history and acknowledgment workflow
- Active alerts panel on dashboard

### Security

- AES-256-GCM encryption for stored credentials
- JWT-based session management
- Role-based access control (Owner, Admin, Member)
- Organization invite system with secure tokens

### Rate Limiting

- Per-server request throttling
- Configurable limits per minute
- 429 responses with retry-after header

### Infrastructure

- Docker Compose for local development
- Kubernetes manifests for production deployment
- TimescaleDB with automatic data retention policies
- Health check background jobs

### Developer Experience

- MCP server examples (API Key, Bearer Token, OAuth)
- Load testing script with configurable profiles
- Alert testing script
- Comprehensive API documentation

## [0.1.0] Initial Release

- Basic MCP proxy functionality
- Server management CRUD
- PostgreSQL database schema
