# mcpx Kubernetes Deployment

Deploy mcpx to any Kubernetes cluster.

## Prerequisites

- Kubernetes cluster (kind, k3s, EKS, GKE, AKS, etc.)
- kubectl configured
- Docker images built and accessible

## Quick Start

### 1. Create secrets file

```bash
cp k8s/base/secrets.yaml k8s/overlays/local/secrets.local.yaml
```

Edit `secrets.local.yaml` with your OAuth credentials from your `.env` file.

### 2. Build and load images (for kind)

```bash
# Build images
docker build --target web -t mcpx-backend:latest ./backend
docker build --target worker -t mcpx-worker:latest ./backend
docker build -t mcpx-frontend:latest ./frontend

# Load into kind
kind load docker-image mcpx-backend:latest --name local
kind load docker-image mcpx-worker:latest --name local
kind load docker-image mcpx-frontend:latest --name local
```

### 3. Deploy

```bash
kubectl apply -k k8s/overlays/local
```

### 4. Configure OAuth Providers

Add these redirect URIs in your OAuth provider consoles:

| Provider | Redirect URI |
|----------|--------------|
| Google | `http://mcpx.127.0.0.1.nip.io/api/auth/google/callback` |
| GitHub | `http://mcpx.127.0.0.1.nip.io/api/auth/github/callback` |

### 5. Access

Open: http://mcpx.127.0.0.1.nip.io

---

## Optional: HTTPS (for Azure OAuth)

Azure requires HTTPS. Use the `local-https` overlay with cert-manager.

### 1. Install cert-manager

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml
kubectl wait --for=condition=Available deployment --all -n cert-manager --timeout=120s
```

### 2. Deploy with HTTPS

```bash
kubectl apply -k k8s/overlays/local-https
```

### 3. Configure Microsoft OAuth

Add in Azure Portal:
```
https://mcpx.127.0.0.1.nip.io/api/auth/microsoft/callback
```

### 4. Accept self-signed certificate

Access https://mcpx.127.0.0.1.nip.io and accept the browser warning.

---

## Structure

```
k8s/
├── base/                    # Base manifests
│   ├── backend/             # Web and worker deployments
│   ├── frontend/            # Frontend deployment
│   ├── postgres/            # PostgreSQL StatefulSet
│   └── mcp-servers/         # Example MCP servers
└── overlays/
    ├── local/               # HTTP (default)
    ├── local-https/         # HTTPS with cert-manager (for Azure)
    └── production/          # For cloud providers
```

## Production Notes

- Use Let's Encrypt instead of self-signed certificates
- Use sealed-secrets or external-secrets for secret management
- Configure HPA for auto-scaling
- Use managed PostgreSQL (RDS, Cloud SQL, etc.)
