# Environment Setup Guide

Complete guide for setting up mcpx in different environments.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [OAuth Provider Setup](#oauth-provider-setup)
3. [Option A: Docker Compose (Recommended for Development)](#option-a-docker-compose-recommended-for-development)
4. [Option B: Kubernetes Local (HTTPS Required)](#option-b-kubernetes-local-https-required)
5. [Option C: Production Deployment](#option-c-production-deployment)

---

## Prerequisites

### Required Software

| Software | Version | Docker Compose | K8s Local | Production |
|----------|---------|:--------------:|:---------:|:----------:|
| Docker | 24+ | ✅ | ✅ | ✅ |
| Docker Compose | 2.20+ | ✅ | ❌ | ❌ |
| kubectl | 1.28+ | ❌ | ✅ | ✅ |
| kind | 0.20+ | ❌ | ✅ | ❌ |
| Helm | 3.12+ | ❌ | Optional | Optional |

### Verify Installation

```bash
docker --version
docker compose version
kubectl version --client
kind version
```

---

## OAuth Provider Setup

Configure OAuth providers before deploying. You'll need credentials from at least one provider.

### Step 1: Google OAuth

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Create or select a project
3. Click **Create Credentials** → **OAuth 2.0 Client ID**
4. Configure consent screen if prompted
5. Application type: **Web application**
6. Add **Authorized JavaScript origins**:
   - Docker Compose: `http://localhost:3000`
   - K8s Local: `https://mcpx.127.0.0.1.nip.io`
7. Add **Authorized redirect URIs**:
   - Docker Compose: `http://localhost:8080/api/auth/google/callback`
   - K8s Local: `https://mcpx.127.0.0.1.nip.io/api/auth/google/callback`
8. Copy **Client ID** and **Client Secret**

### Step 2: GitHub OAuth

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click **New OAuth App**
3. Fill in:
   - **Application name**: mcpx
   - **Homepage URL**: `http://localhost:3000` (or your domain)
   - **Authorization callback URL**:
     - Docker Compose: `http://localhost:8080/api/auth/github/callback`
     - K8s Local: `https://mcpx.127.0.0.1.nip.io/api/auth/github/callback`
4. Click **Register application**
5. Copy **Client ID** and generate **Client Secret**

### Step 3: Microsoft Azure OAuth

> **Note**: Azure requires HTTPS for redirect URIs (except localhost)

1. Go to [Azure Portal - App Registrations](https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade)
2. Click **New registration**
3. Fill in:
   - **Name**: mcpx
   - **Supported account types**: Accounts in any organizational directory and personal Microsoft accounts
4. Click **Register**
5. Go to **Authentication** → **Add a platform** → **Web**
6. Add redirect URIs:
   - Docker Compose: `http://localhost:8080/api/auth/microsoft/callback`
   - K8s Local: `https://mcpx.127.0.0.1.nip.io/api/auth/microsoft/callback`
7. Go to **Certificates & secrets** → **New client secret**
8. Copy **Application (client) ID** and **Secret Value**

---

## Option A: Docker Compose (Recommended for Development)

Best for local development. Simple setup, no HTTPS required.

### Step 1: Clone Repository

```bash
git clone https://github.com/your-org/mcpx.git
cd mcpx
```

### Step 2: Create Environment File

```bash
cp .env.example .env
```

Edit `.env` with your OAuth credentials:

```bash
# Database (auto-created by docker-compose)
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/mcpx

# JWT Secret (generate with: openssl rand -hex 32)
JWT_SECRET=your-32-char-secret-here

# Google OAuth
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# GitHub OAuth
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret

# Microsoft OAuth (optional)
MICROSOFT_CLIENT_ID=your-microsoft-client-id
MICROSOFT_CLIENT_SECRET=your-microsoft-client-secret

# URLs
FRONTEND_URL=http://localhost:3000
BASE_URL=http://localhost:8080
```

### Step 3: Build Images

```bash
docker compose build
```

### Step 4: Start Services

```bash
docker compose up -d
```

### Step 5: Verify Services

```bash
# Check all containers are running
docker compose ps

# Check logs
docker compose logs -f backend
```

### Step 6: Access Application

Open http://localhost:3000

### Troubleshooting

```bash
# Restart services
docker compose restart

# View logs
docker compose logs -f

# Reset database
docker compose down -v
docker compose up -d
```

---

## Option B: Kubernetes Local (HTTPS Required)

For testing Kubernetes deployments locally. **HTTPS is required** for Microsoft OAuth.

### Step 1: Create Kind Cluster

```bash
# Create cluster with port mappings
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

### Step 2: Install Ingress NGINX

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

# Wait for ingress to be ready
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=120s
```

### Step 3: Install cert-manager

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml

# Wait for cert-manager
kubectl wait --for=condition=Available deployment --all -n cert-manager --timeout=120s
```

### Step 4: Build and Load Docker Images

```bash
# Build images
docker build --target web -t mcpx-backend:latest ./backend
docker build --target worker -t mcpx-worker:latest ./backend
docker build -t mcpx-frontend:latest ./frontend

# Build MCP example servers
docker build -t mcpx-weather:latest ./mcp-servers-examples/weather
docker build -t mcpx-utilities:latest ./mcp-servers-examples/utilities-mcp
docker build -t mcpx-filesystem:latest ./mcp-servers-examples/filesystem-mcp

# Load into kind
kind load docker-image mcpx-backend:latest --name local
kind load docker-image mcpx-worker:latest --name local
kind load docker-image mcpx-frontend:latest --name local
kind load docker-image mcpx-weather:latest --name local
kind load docker-image mcpx-utilities:latest --name local
kind load docker-image mcpx-filesystem:latest --name local
```

### Step 5: Create Secrets File

```bash
# Create secrets file from template
cp k8s/base/secrets.yaml k8s/overlays/local-https/secrets.local.yaml
```

Edit `k8s/overlays/local-https/secrets.local.yaml`:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: mcpx-secrets
  namespace: mcpx
type: Opaque
stringData:
  POSTGRES_USER: "postgres"
  POSTGRES_PASSWORD: "your-db-password"
  JWT_SECRET: "your-32-char-jwt-secret"
  GOOGLE_CLIENT_ID: "your-google-client-id"
  GOOGLE_CLIENT_SECRET: "your-google-client-secret"
  GITHUB_CLIENT_ID: "your-github-client-id"
  GITHUB_CLIENT_SECRET: "your-github-client-secret"
  MICROSOFT_CLIENT_ID: "your-microsoft-client-id"
  MICROSOFT_CLIENT_SECRET: "your-microsoft-client-secret"
```

### Step 6: Deploy to Kubernetes

```bash
# Apply manifests with HTTPS overlay
kubectl apply -k k8s/overlays/local-https

# Deploy MCP example servers
kubectl apply -k k8s/base/mcp-servers
```

### Step 7: Verify Deployment

```bash
# Check all pods are running
kubectl get pods -n mcpx

# Check certificate is ready
kubectl get certificate -n mcpx

# Check ingress
kubectl get ingress -n mcpx
```

### Step 8: Access Application

Open https://mcpx.127.0.0.1.nip.io

> **Note**: Accept the self-signed certificate warning in your browser.

### Troubleshooting

```bash
# Check pod logs
kubectl logs -n mcpx -l app=mcpx-web -f

# Check events
kubectl get events -n mcpx --sort-by='.lastTimestamp'

# Restart deployment
kubectl rollout restart deployment mcpx-web -n mcpx

# Delete and recreate
kubectl delete -k k8s/overlays/local-https
kubectl apply -k k8s/overlays/local-https
```

---

## Option C: Production Deployment

Recommendations for production environments on cloud providers.

### Architecture Overview

```mermaid
flowchart TB
    subgraph cloud["Cloud Provider"]
        DNS["DNS<br/>(Route53, CloudDNS)"]
        TLS["SSL/TLS<br/>(ACM, Let's Encrypt)"]
        
        subgraph k8s["Kubernetes Cluster (EKS, GKE, AKS)"]
            ingress["Ingress Controller"]
            web["mcpx-web<br/>Replicas: 3+"]
            worker["mcpx-worker<br/>Replicas: 2+"]
            frontend["mcpx-frontend"]
        end
        
        subgraph paas["Managed Database (PaaS)"]
            db[("PostgreSQL<br/>(RDS, Cloud SQL, Azure DB)<br/>• Automated backups<br/>• High availability<br/>• Encryption at rest")]
        end
    end
    
    user((User)) --> DNS
    DNS --> TLS
    TLS --> ingress
    ingress --> frontend
    ingress --> web
    web --> db
    worker --> db
```

### Step 1: Set Up Managed Database

#### AWS RDS

```bash
aws rds create-db-instance \
  --db-instance-identifier mcpx-db \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --master-username postgres \
  --master-user-password YOUR_SECURE_PASSWORD \
  --allocated-storage 20 \
  --backup-retention-period 7 \
  --storage-encrypted
```

#### GCP Cloud SQL

```bash
gcloud sql instances create mcpx-db \
  --database-version=POSTGRES_15 \
  --tier=db-f1-micro \
  --region=us-central1 \
  --root-password=YOUR_SECURE_PASSWORD
```

#### Azure Database

```bash
az postgres server create \
  --resource-group mcpx-rg \
  --name mcpx-db \
  --location eastus \
  --admin-user postgres \
  --admin-password YOUR_SECURE_PASSWORD \
  --sku-name B_Gen5_1
```

### Step 2: Configure Secrets Management

> **Recommended**: Use your cloud provider's secrets vault for production environments.

#### Cloud Provider Vaults (Recommended)

| Provider | Service | Documentation |
|----------|---------|---------------|
| AWS | Secrets Manager | [docs](https://aws.amazon.com/secrets-manager/) |
| GCP | Secret Manager | [docs](https://cloud.google.com/secret-manager) |
| Azure | Key Vault | [docs](https://azure.microsoft.com/products/key-vault/) |

**Benefits:**
- Automatic rotation of secrets
- Audit logging of access
- Fine-grained IAM permissions
- Encryption at rest and in transit
- No secrets in version control

#### Integration with Kubernetes

Use [External Secrets Operator](https://external-secrets.io/) to sync secrets from your vault to Kubernetes:

```bash
# Install External Secrets Operator
helm repo add external-secrets https://charts.external-secrets.io
helm install external-secrets external-secrets/external-secrets -n external-secrets --create-namespace
```

**AWS Secrets Manager example:**
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: mcpx-secrets
  namespace: mcpx
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: ClusterSecretStore
  target:
    name: mcpx-secrets
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: mcpx/database-url
    - secretKey: JWT_SECRET
      remoteRef:
        key: mcpx/jwt-secret
    - secretKey: GOOGLE_CLIENT_ID
      remoteRef:
        key: mcpx/google-client-id
    - secretKey: GOOGLE_CLIENT_SECRET
      remoteRef:
        key: mcpx/google-client-secret
```

#### Alternative: Kubernetes Secrets (Not Recommended for Production)

Only if vault integration is not possible:

```bash
kubectl create secret generic mcpx-secrets \
  --from-literal=DATABASE_URL="postgresql://user:pass@host:5432/mcpx" \
  --from-literal=JWT_SECRET="..." \
  --dry-run=client -o yaml | kubectl apply -f -
```

> ⚠️ **Warning**: This approach stores secrets in etcd. Enable [encryption at rest](https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/) if using this method.

### Step 3: Create Production Overlay

Create `k8s/overlays/production/kustomization.yaml`:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base
  - ingress.yaml
  - certificate.yaml

patchesStrategicMerge:
  - configmap.yaml

# Use HPA for auto-scaling
patches:
  - patch: |-
      - op: replace
        path: /spec/replicas
        value: 3
    target:
      kind: Deployment
      name: mcpx-web
```

### Step 4: Configure TLS with Let's Encrypt

Create `k8s/overlays/production/cluster-issuer.yaml`:

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

### Step 5: Deploy

```bash
# Apply production overlay
kubectl apply -k k8s/overlays/production

# Verify
kubectl get pods -n mcpx
kubectl get certificate -n mcpx
```

### Production Checklist

- [ ] Managed database with automated backups
- [ ] Secrets stored in secrets manager (not in git)
- [ ] TLS with Let's Encrypt or cloud-managed certificates
- [ ] Horizontal Pod Autoscaler (HPA) configured
- [ ] Resource limits set for all containers
- [ ] Liveness and readiness probes configured
- [ ] Logging and monitoring enabled
- [ ] Network policies restricting traffic
- [ ] Regular security updates scheduled

---

## Quick Reference

### Environment Variables

| Variable | Required | Description |
|----------|:--------:|-------------|
| `DATABASE_URL` | ✅ | PostgreSQL connection string |
| `JWT_SECRET` | ✅ | Secret for JWT token signing (32+ chars) |
| `GOOGLE_CLIENT_ID` | ⚠️ | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | ⚠️ | Google OAuth client secret |
| `GITHUB_CLIENT_ID` | ⚠️ | GitHub OAuth client ID |
| `GITHUB_CLIENT_SECRET` | ⚠️ | GitHub OAuth client secret |
| `MICROSOFT_CLIENT_ID` | ⚠️ | Microsoft OAuth client ID |
| `MICROSOFT_CLIENT_SECRET` | ⚠️ | Microsoft OAuth client secret |
| `FRONTEND_URL` | ❌ | Frontend URL (default: http://localhost:3000) |
| `BASE_URL` | ❌ | Backend URL for OAuth redirects |
| `RUST_LOG` | ❌ | Log level (default: info) |

⚠️ = At least one OAuth provider required

### OAuth Redirect URIs

| Provider | Docker Compose | K8s Local (HTTPS) |
|----------|----------------|-------------------|
| Google | `http://localhost:8080/api/auth/google/callback` | `https://mcpx.127.0.0.1.nip.io/api/auth/google/callback` |
| GitHub | `http://localhost:8080/api/auth/github/callback` | `https://mcpx.127.0.0.1.nip.io/api/auth/github/callback` |
| Microsoft | `http://localhost:8080/api/auth/microsoft/callback` | `https://mcpx.127.0.0.1.nip.io/api/auth/microsoft/callback` |

### Useful Commands

```bash
# Docker Compose
docker compose up -d          # Start
docker compose down           # Stop
docker compose logs -f        # Logs
docker compose ps             # Status

# Kubernetes
kubectl get pods -n mcpx      # Pod status
kubectl logs -n mcpx -l app=mcpx-web -f  # Logs
kubectl rollout restart deployment mcpx-web -n mcpx  # Restart
kubectl get events -n mcpx --sort-by='.lastTimestamp'  # Events
```
