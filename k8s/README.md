# mcpx Kubernetes Deployment

Deploy mcpx to any Kubernetes cluster.

## Prerequisites

- Kubernetes cluster (kind, k3s, EKS, GKE, AKS, etc.)
- kubectl configured
- Docker images built and accessible

## Quick Start

### 1. Create secrets file (for real values)

```bash
cp k8s/base/secrets.yaml k8s/base/secrets.local.yaml
```

Edit `k8s/base/secrets.local.yaml` with your actual values:
- `JWT_SECRET`: 32+ character secret
- `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET`: For OAuth login
- `POSTGRES_PASSWORD`: Database password

Then apply before deploying:
```bash
kubectl apply -f k8s/base/secrets.local.yaml
```
- `JWT_SECRET`: 32+ character secret
- `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET`: For OAuth login
- `POSTGRES_PASSWORD`: Database password

### 2. Build and load images (for kind)

```bash
# Build images
docker build --target web -t mcpx-backend:latest ./backend
docker build --target worker -t mcpx-worker:latest ./backend
docker build -t mcpx-frontend:latest ./frontend

# Load into kind (skip if using registry)
kind load docker-image mcpx-backend:latest --name local
kind load docker-image mcpx-worker:latest --name local
kind load docker-image mcpx-frontend:latest --name local
```

### 3. Deploy

```bash
kubectl apply -k k8s/overlays/local
```

### 4. Access

Add to `/etc/hosts`:
```
127.0.0.1 mcpx.local
```

Access: http://mcpx.local

## Structure

```
k8s/
├── base/                    # Base manifests
│   ├── kustomization.yaml
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml.example # Copy to secrets.yaml
│   ├── postgres/
│   ├── backend/
│   └── frontend/
└── overlays/
    ├── local/              # For kind/local clusters
    └── production/         # For cloud providers
```

## Production Notes

- Use sealed-secrets or external-secrets for secret management
- Configure HPA for auto-scaling
- Use managed PostgreSQL (RDS, Cloud SQL, etc.)
