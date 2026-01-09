<p align="center">
  <img src="frontend/src/assets/mcpx-logo.svg" alt="mcpx" width="80" height="80">
</p>

<h1 align="center">mcpx</h1>

<p align="center">
  <strong>The MCP Gateway for Enterprise</strong>
</p>

<p align="center">
  Bridge the gap between AI agents and your data.<br>
  Secure, scalable, production-grade.
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#deployment">Deployment</a> •
  <a href="#configuration">Configuration</a> •
  <a href="USECASES.md">Use Cases</a> •
  <a href="#api-reference">API</a> •
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-Apache%202.0-blue.svg">
  <img alt="Status" src="https://img.shields.io/badge/status-incubating-purple.svg">
</p>

> **Note:** This is an incubating project maintained by [TM Dev Lab](https://www.tmdevlab.com). While functional, it may contain bugs or incomplete features. If you encounter issues, please report them via [GitHub Issues](https://github.com/thiagomendes/mcpx/issues) or consider contributing a fix following our [Contributing Guidelines](CONTRIBUTING.md).

## Overview

mcpx is a self-hosted gateway for the Model Context Protocol (MCP). It enables organizations to securely connect AI agents to MCP servers while maintaining full control over access, credentials, and observability.

> **Scope:** mcpx focuses exclusively on **MCP Tools**, which represent the majority of protocol usage. Resources and Prompts are not currently supported.

### Key Capabilities

| Capability | Description |
|------------|-------------|
| **Proxy Layer** | Route MCP traffic through a central gateway with credential injection |
| **Tool Governance** | Allowlist or blocklist specific tools per server |
| **Virtual Gateways** | Aggregate multiple servers into a single endpoint |
| **Observability** | Audit logs, real-time metrics, and alerting |
| **Authentication** | OAuth, API Keys, Bearer Tokens, and Service Accounts |
| **Multi-tenancy** | Isolated organizations with role-based access control |

For detailed workflows with sequence diagrams and RBAC matrix, see [Use Cases](USECASES.md).
### Architecture

```mermaid
flowchart LR
    subgraph Clients
        CD["Claude Desktop"]
        VS["VS Code"]
        Agent["AI Agents"]
    end

    subgraph mcpx["mcpx Gateway"]
        Proxy["MCP Proxy"]
        Auth["Auth"]
        Gov["Governance"]
        Audit["Audit Log"]
    end

    subgraph Servers["MCP Servers"]
        S1["Finance MCP"]
        S2["HR MCP"]
        S3["External MCP"]
    end

    CD --> Proxy
    VS --> Proxy
    Agent --> Proxy
    Proxy --> Auth
    Auth --> Gov
    Gov --> S1
    Gov --> S2
    Gov --> S3
    Proxy --> Audit

    style Proxy fill:#6366f1,stroke:#4f46e5,color:#fff
    style Auth fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style Gov fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style Audit fill:#06b6d4,stroke:#0891b2,color:#fff
```

## Features

### Connect Instantly

Point to any MCP server via URL. mcpx handles credential injection and protocol compliance automatically.

### Tool Governance

Control which tools AI agents can access. Define allowlists or blocklists per server, and apply prefixes to tool names for namespace isolation.

### Full Observability

Every MCP request is logged with timestamps, latency, tool names, and full request/response payloads. Query metrics by time range, server, or tool.

### Rate Limiting

Protect your MCP servers with per-server rate limits. Configurable requests per minute with automatic 429 responses.

### Flexible Authentication

Support for multiple authentication methods:

| Method | Use Case |
|--------|----------|
| OAuth | User authentication via Google, GitHub, or Microsoft |
| Personal Access Tokens | Programmatic access for users |
| Service Accounts | Machine-to-machine communication |
| API Keys and Bearer Tokens | Credential injection to upstream servers |

### Gateway Aggregation

Combine multiple MCP servers into a single endpoint. Connect Claude Desktop to one URL and access all your tools seamlessly.

## See it in Action

Watch some of the key features in action:

<table>
  <tr>
    <td align="center">
      <a href="https://www.youtube.com/watch?v=b0scYQ3qIFw">
        <img src="https://img.youtube.com/vi/b0scYQ3qIFw/mqdefault.jpg" width="200"><br>
        <b>01. Create Server</b>
      </a>
    </td>
    <td align="center">
      <a href="https://www.youtube.com/watch?v=CG3lEmAj6AE">
        <img src="https://img.youtube.com/vi/CG3lEmAj6AE/mqdefault.jpg" width="200"><br>
        <b>02. Create Gateway</b>
      </a>
    </td>
    <td align="center">
      <a href="https://www.youtube.com/watch?v=vCbqk9A2AEg">
        <img src="https://img.youtube.com/vi/vCbqk9A2AEg/mqdefault.jpg" width="200"><br>
        <b>03. Observability</b>
      </a>
    </td>
    <td align="center">
      <a href="https://www.youtube.com/watch?v=AIQdCAtVQlQ">
        <img src="https://img.youtube.com/vi/AIQdCAtVQlQ/mqdefault.jpg" width="200"><br>
        <b>04. Alerts</b>
      </a>
    </td>
  </tr>
</table>

## Example MCP Servers

The project includes example MCP servers in the `mcp-servers-examples/` directory for testing different authentication methods. These servers start automatically with Docker Compose.

| Server | Auth Method | Internal URL |
|--------|-------------|--------------|
| weather-mcp | API Key | `http://weather-mcp:8000/mcp` |
| utilities-mcp | Bearer Token | `http://utilities-mcp:8000/mcp` |
| filesystem-mcp | OAuth Client Credentials | `http://filesystem-mcp:8000/mcp` |

### Test Credentials

| Server | Credential |
|--------|------------|
| weather-mcp | Header: `X-API-Key: weather-api-key-12345` |
| utilities-mcp | Header: `Authorization: Bearer utilities-token-secret-67890` |
| filesystem-mcp | Client ID: `filesystem-client`, Secret: `filesystem-secret-abc123` |

## Quick Start

### Prerequisites

| Software | Version |
|----------|---------|
| Docker | 24.x or later |
| Docker Compose | 2.20 or later |

### Clone and Configure

```bash
git clone https://github.com/thiagomendes/mcpx.git
cd mcpx

cp .env.example .env
```

Edit `.env` with your OAuth credentials. At minimum, configure one OAuth provider:

```bash
JWT_SECRET=your-32-character-secret-here

GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret

FRONTEND_URL=http://localhost:3000
BASE_URL=http://localhost:8080
```

### Build and Start

```bash
docker compose build
docker compose up -d
```

> **Note:** First build may take a few minutes. Subsequent builds are cached.

### Verify Installation

```bash
docker compose ps

curl http://localhost:8080/api/health
```

Open http://localhost:3000 to access the dashboard.

## Deployment

mcpx supports multiple deployment options:

| Environment | Method | Documentation |
|-------------|--------|---------------|
| Local Development | Docker Compose | [Quick Start](#quick-start) |
| Local Kubernetes | Kind with HTTPS | [Kubernetes Local](#kubernetes-local) |
| Production | Kubernetes with managed services | [Production Deployment](#production-deployment) |

### Kubernetes Local

For testing Kubernetes deployments locally with HTTPS support.

#### Prerequisites

| Software | Version |
|----------|---------|
| kubectl | 1.28 or later |
| kind | 0.20 or later |

#### Create Cluster

```bash
cat <<EOF | kind create cluster --name local --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
EOF
```

#### Install Dependencies

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=120s

kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml

kubectl wait --for=condition=Available deployment --all -n cert-manager --timeout=120s
```

#### Build and Load Images

```bash
docker build --target web -t mcpx-backend:latest ./backend
docker build --target worker -t mcpx-worker:latest ./backend
docker build -t mcpx-frontend:latest ./frontend

kind load docker-image mcpx-backend:latest --name local
kind load docker-image mcpx-worker:latest --name local
kind load docker-image mcpx-frontend:latest --name local
```

#### Configure Secrets

```bash
cp k8s/base/secrets.yaml k8s/overlays/local-https/secrets.local.yaml
```

Edit `k8s/overlays/local-https/secrets.local.yaml` with your credentials.

#### Deploy

```bash
kubectl apply -k k8s/overlays/local-https
```

Access the application at https://mcpx.127.0.0.1.nip.io

> The domain `mcpx.127.0.0.1.nip.io` resolves to 127.0.0.1 automatically. No hosts file modification required.

### Production Deployment

For production environments on cloud providers (AWS, GCP, Azure).

#### Architecture

```mermaid
flowchart TB
    subgraph Internet
        LB["Load Balancer + TLS"]
    end

    subgraph K8s["Kubernetes Cluster"]
        direction TB
        subgraph Services
            Web["Web<br/>3+ replicas"]
            Worker["Worker<br/>2+ replicas"]
        end
    end

    subgraph Data["Managed Services"]
        DB[("PostgreSQL<br/>TimescaleDB")]
    end

    LB --> Web
    LB --> Worker
    Web --> DB
    Worker --> DB

    style LB fill:#6366f1,stroke:#4f46e5,color:#fff
    style Web fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style Worker fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style DB fill:#06b6d4,stroke:#0891b2,color:#fff
```

#### Managed Database

mcpx requires PostgreSQL with the TimescaleDB extension.

| Provider | Managed PostgreSQL | TimescaleDB Support |
|----------|-------------------|---------------------|
| AWS | RDS for PostgreSQL | Not available |
| GCP | Cloud SQL | Not available |
| Azure | Flexible Server | Available |

For AWS and GCP, use [Timescale Cloud](https://www.timescale.com/cloud) or run TimescaleDB in Kubernetes.

#### Secrets Management

Use your cloud provider's secrets management service:

| Provider | Service |
|----------|---------|
| AWS | Secrets Manager |
| GCP | Secret Manager |
| Azure | Key Vault |

Integrate with Kubernetes using [External Secrets Operator](https://external-secrets.io/).

#### Deploy

```bash
kubectl apply -k k8s/overlays/production
```

#### Production Checklist

| Item | Status |
|------|--------|
| Managed database with backups | Required |
| Secrets in vault (not in git) | Required |
| TLS via cloud provider | Required |
| Horizontal Pod Autoscaler | Recommended |
| Resource limits configured | Recommended |
| Network policies | Recommended |

## Configuration

### Environment Variables

> **Note:** When using Docker Compose, most variables have sensible defaults. Only OAuth credentials and JWT_SECRET are required.

| Variable | Required | Description |
|----------|:--------:|-------------|
| `JWT_SECRET` | Yes | Secret for JWT signing (32+ characters) |
| `GOOGLE_CLIENT_ID` | * | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | * | Google OAuth client secret |
| `GITHUB_CLIENT_ID` | * | GitHub OAuth client ID |
| `GITHUB_CLIENT_SECRET` | * | GitHub OAuth client secret |
| `MICROSOFT_CLIENT_ID` | * | Microsoft OAuth client ID |
| `MICROSOFT_CLIENT_SECRET` | * | Microsoft OAuth client secret |
| `DATABASE_URL` | ** | PostgreSQL connection string |
| `FRONTEND_URL` | No | Frontend URL (default: http://localhost:3000) |
| `BASE_URL` | No | Backend URL for OAuth callbacks |
| `RUST_LOG` | No | Log level (default: info) |
| `ENCRYPTION_KEY` | No | AES encryption key (defaults to JWT_SECRET) |

\* At least one OAuth provider is required.

\*\* Required for standalone deployment. Docker Compose sets this automatically.

### OAuth Provider Setup

#### Google

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Create OAuth Client ID
3. Configure redirect URI: `{BASE_URL}/api/auth/google/callback`

#### GitHub

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Create new OAuth App
3. Configure callback URL: `{BASE_URL}/api/auth/github/callback`

#### Microsoft

1. Go to [Azure Portal App Registrations](https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade)
2. Create new registration
3. Add redirect URI: `{BASE_URL}/api/auth/microsoft/callback`

## API Reference

### Authentication Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/auth/{provider}` | Initiate OAuth flow |
| GET | `/api/auth/{provider}/callback` | OAuth callback |
| POST | `/api/auth/logout` | End session |
| GET | `/api/auth/me` | Current user info |

### Server Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/servers` | List servers |
| POST | `/api/servers` | Create server |
| GET | `/api/servers/{name}` | Get server |
| PUT | `/api/servers/{name}` | Update server |
| DELETE | `/api/servers/{name}` | Delete server |

### Gateway Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/gateways` | List gateways |
| POST | `/api/gateways` | Create gateway |
| GET | `/api/gateways/{slug}` | Get gateway |
| PUT | `/api/gateways/{slug}` | Update gateway |
| DELETE | `/api/gateways/{slug}` | Delete gateway |

### MCP Proxy

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/mcp/{org_slug}/{server}` | Proxy MCP request to server |
| POST | `/mcp/{org_slug}/gw/{gateway}` | Proxy MCP request to gateway |

### Metrics and Audit

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/metrics/query` | Query time-series metrics |
| GET | `/api/metrics/summary` | Percentile summary |
| GET | `/api/audit` | List audit logs |
| GET | `/api/audit/{id}` | Audit log details |

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust with Axum |
| Frontend | Vue 3 with TypeScript |
| Database | PostgreSQL 15 with TimescaleDB |
| Authentication | OAuth with JWT |
| Container Runtime | Docker |
| Orchestration | Kubernetes |

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

1. Fork and clone the repository
2. Create a feature branch
3. Make changes following coding standards
4. Submit a pull request

For AI-assisted development guidelines, see [AGENT.md](AGENT.md).

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Support

For questions or issues, contact mail.thiagomendes@gmail.com or open an issue on GitHub.

Copyright 2025 TM Dev Lab
