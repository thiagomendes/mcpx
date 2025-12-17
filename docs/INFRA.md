# mcpx - Infrastructure Deployment Guide

**Purpose:** Internal documentation for deploying the mcpx SaaS platform.
**Target:** Cloud-agnostic Kubernetes (K8s)
**Development:** Docker Desktop K8s (local)
**Production:** Any managed K8s (AKS, EKS, GKE, DOKS)

---

## 📖 Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development Setup](#local-development-setup)
- [Docker Desktop K8s Setup](#docker-desktop-k8s-setup)
- [Deploy to Local K8s](#deploy-to-local-k8s)
- [Dockerfiles](#dockerfiles)
- [Kubernetes Manifests](#kubernetes-manifests)
- [Cloud Deployment Options](#cloud-deployment-options)

---

# PREREQUISITES

```bash
# Node.js 20+
node --version  # v20.x.x

# pnpm
pnpm --version  # 9.x.x

# Docker Desktop (with K8s enabled)
docker --version
kubectl version --client

# Skaffold (optional, for hot reload)
skaffold version
```

---

# LOCAL DEVELOPMENT SETUP

## 1. Initialize Frontend (Vue 3 + Vite)

```bash
cd mcpx

# Create frontend directory and initialize Vue 3
mkdir frontend
cd frontend
pnpm create vue@latest . --typescript

# During setup, select:
# ✅ TypeScript
# ✅ Vue Router
# ✅ Pinia (state management)
# ❌ Vitest (we'll add later)
# ❌ Playwright (we'll add later)

# Install dependencies
pnpm install

# Add UI dependencies
pnpm add axios
pnpm add -D tailwindcss postcss autoprefixer
pnpm add @headlessui/vue @heroicons/vue

# Initialize Tailwind CSS
pnpm dlx tailwindcss init -p
```

**Configure Tailwind** (`frontend/tailwind.config.js`):

```js
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#6366f1',    // Indigo (TM Dev Lab)
        secondary: '#ec4899',  // Pink
      },
    },
  },
  plugins: [],
}
```

## 2. Initialize Backend (Rust + Axum)

```bash
cd mcpx

# Create backend directory and initialize Rust project
mkdir backend
cd backend
cargo init --name mcpx-backend

# Create migrations directory
mkdir migrations
```

**Update `backend/Cargo.toml`:**

```toml
[package]
name = "mcpx-backend"
version = "1.0.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono", "migrate"] }

# Authentication
oauth2 = "4.4"
jsonwebtoken = "9.2"
reqwest = { version = "0.11", features = ["json"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Install SQLx CLI:**

```bash
# Install sqlx-cli for migrations
cargo install sqlx-cli --no-default-features --features postgres
```

## 3. Build Docker Images

```bash
# Build frontend image
cd mcpx/frontend
docker build -t mcpx-frontend:latest .

# Build backend image
cd mcpx/backend
docker build -t mcpx-backend:latest .
```

---

# DOCKER DESKTOP K8S SETUP

## Enable Kubernetes in Docker Desktop

1. **Open Docker Desktop**
2. **Go to Settings → Kubernetes**
3. **Check "Enable Kubernetes"**
4. **Click "Apply & Restart"**
5. **Wait ~2-3 minutes** for K8s to start

## Verify K8s is Running

```bash
kubectl cluster-info
# Output: Kubernetes control plane is running at https://kubernetes.docker.internal:6443

kubectl get nodes
# Output: docker-desktop   Ready    control-plane   ...
```

## Install K8s Dashboard (Optional)

```bash
# Install dashboard
kubectl apply -f https://raw.githubusercontent.com/kubernetes/dashboard/v2.7.0/aio/deploy/recommended.yaml

# Create admin user
kubectl create serviceaccount dashboard-admin -n kubernetes-dashboard
kubectl create clusterrolebinding dashboard-admin \
  --clusterrole=cluster-admin \
  --serviceaccount=kubernetes-dashboard:dashboard-admin

# Get access token
kubectl -n kubernetes-dashboard create token dashboard-admin

# Start proxy
kubectl proxy
# Open: http://localhost:8001/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/
```

---

# DEPLOY TO LOCAL K8S

## Quick Deploy (10 min)

```bash
# Create namespace
kubectl create namespace mcpx

# Create secrets (Google OAuth)
kubectl create secret generic google-oauth -n mcpx \
  --from-literal=client-id='your-google-client-id' \
  --from-literal=client-secret='your-google-client-secret'

# Create secrets (JWT)
kubectl create secret generic jwt-secret -n mcpx \
  --from-literal=secret=$(openssl rand -base64 32)

# Apply all K8s manifests
kubectl apply -f k8s/ -n mcpx

# Watch pods starting
kubectl get pods -n mcpx -w
```

## Verify Deployment

```bash
# Check all resources
kubectl get all -n mcpx

# Expected output:
# - deployment/frontend (1/1 ready)
# - deployment/backend (1/1 ready)
# - deployment/postgres (1/1 ready)
# - deployment/redis (1/1 ready)
# - service/frontend (LoadBalancer)
# - service/backend (ClusterIP)
# - service/postgres
# - service/redis
```

## Access Application

```bash
# Get LoadBalancer external IP (may take 1-2 minutes)
kubectl get svc frontend -n mcpx

# For Docker Desktop, LoadBalancer uses localhost
# Open browser: http://localhost

# Or use port-forward if LoadBalancer doesn't work
kubectl port-forward -n mcpx svc/frontend 8080:80

# Open browser: http://localhost:8080
```

## View Logs

```bash
# Frontend logs (Vue + Nginx)
kubectl logs -n mcpx -l app=frontend -f

# Backend logs (Rust + Axum)
kubectl logs -n mcpx -l app=backend -f

# PostgreSQL logs
kubectl logs -n mcpx -l app=postgres -f

# Redis logs
kubectl logs -n mcpx -l app=redis -f

# All logs (follow)
kubectl logs -n mcpx --all-containers=true -f
```

## Google OAuth Setup (10 min)

1. Go to https://console.cloud.google.com
2. Create project: "mcpx Dev"
3. Enable Google+ API
4. Create OAuth credentials:
   - Type: Web application
   - Authorized JavaScript origins: `http://localhost`
   - Redirect URI: `http://localhost/api/auth/google/callback`
5. Update K8s Secret:
   ```bash
   kubectl delete secret google-oauth -n mcpx
   kubectl create secret generic google-oauth -n mcpx \
     --from-literal=client-id='your-actual-client-id' \
     --from-literal=client-secret='your-actual-client-secret'

   # Restart backend pods to pick up new secrets
   kubectl rollout restart deployment/backend -n mcpx
   ```

---

# DOCKERFILES

## Frontend (Vue 3 + Vite)

**Location:** `frontend/Dockerfile`

```dockerfile
# Build stage
FROM node:20-alpine AS builder
WORKDIR /app

# Copy package files
COPY package.json pnpm-lock.yaml* ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile

# Copy source and build
COPY . .
RUN pnpm build

# Production stage with Nginx
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

**Nginx Config:** `frontend/nginx.conf`

```nginx
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy to backend
    location /api/ {
        proxy_pass http://backend:8080/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # Health check
    location /health {
        access_log off;
        return 200 "healthy\n";
    }
}
```

## Backend (Rust + Axum)

**Location:** `backend/Dockerfile`

```dockerfile
# Build stage
FROM rust:1.75-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source and build
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:latest
RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/mcpx-backend ./

EXPOSE 8080
USER 1000:1000
CMD ["./mcpx-backend"]
```

## PostgreSQL & Redis

Using official images:
- **PostgreSQL:** `postgres:15-alpine`
- **Redis:** `redis:7-alpine`

---

# KUBERNETES MANIFESTS

All manifests in `k8s/` directory.

## Namespace (`k8s/00-namespace.yaml`)

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: mcpx
```

## ConfigMap (`k8s/01-configmap.yaml`)

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mcpx-config
  namespace: mcpx
data:
  DATABASE_URL: "postgresql://postgres:postgres@postgres:5432/mcpx"
  REDIS_URL: "redis://redis:6379"
  BACKEND_URL: "http://backend:8080"
  FRONTEND_URL: "http://localhost"
```

## Secrets (`k8s/02-secrets.yaml`)

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: google-oauth
  namespace: mcpx
type: Opaque
stringData:
  client-id: "your-google-client-id"
  client-secret: "your-google-client-secret"
---
apiVersion: v1
kind: Secret
metadata:
  name: jwt-secret
  namespace: mcpx
type: Opaque
stringData:
  secret: "generate-with-openssl-rand-base64-32"
---
apiVersion: v1
kind: Secret
metadata:
  name: postgres-secret
  namespace: mcpx
type: Opaque
stringData:
  password: "postgres"  # Change in production!
```

## PostgreSQL (`k8s/10-postgres.yaml`)

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: mcpx
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 5Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: mcpx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          value: mcpx
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
          subPath: postgres
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: mcpx
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  type: ClusterIP
```

## Redis (`k8s/11-redis.yaml`)

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: redis-pvc
  namespace: mcpx
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: mcpx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        command: ["redis-server"]
        args: ["--appendonly", "yes", "--maxmemory", "256mb", "--maxmemory-policy", "allkeys-lru"]
        volumeMounts:
        - name: redis-storage
          mountPath: /data
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
      volumes:
      - name: redis-storage
        persistentVolumeClaim:
          claimName: redis-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: mcpx
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
  type: ClusterIP
```

## Frontend (`k8s/20-frontend.yaml`)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
  namespace: mcpx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
    spec:
      containers:
      - name: frontend
        image: mcpx-frontend:latest
        imagePullPolicy: Never  # For Docker Desktop local images
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "128Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: frontend
  namespace: mcpx
spec:
  selector:
    app: frontend
  ports:
  - port: 80
    targetPort: 80
  type: LoadBalancer
```

## Backend (`k8s/21-backend.yaml`)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
  namespace: mcpx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
      - name: backend
        image: mcpx-backend:latest
        imagePullPolicy: Never
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            configMapKeyRef:
              name: mcpx-config
              key: DATABASE_URL
        - name: REDIS_URL
          valueFrom:
            configMapKeyRef:
              name: mcpx-config
              key: REDIS_URL
        - name: GOOGLE_CLIENT_ID
          valueFrom:
            secretKeyRef:
              name: google-oauth
              key: client-id
        - name: GOOGLE_CLIENT_SECRET
          valueFrom:
            secretKeyRef:
              name: google-oauth
              key: client-secret
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: backend
  namespace: mcpx
spec:
  selector:
    app: backend
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
```

---

# CLOUD DEPLOYMENT OPTIONS

**Status:** Future - For production deployment after local validation.

Once local K8s development is complete, you can deploy to any cloud provider:

## Option 1: Azure AKS

- **AKS Cluster:** Free control plane + VM costs ($116-167/month for single node)
- **Managed Services:** Azure Database for PostgreSQL + Azure Cache for Redis
- **Pros:** Familiar Azure ecosystem, managed services
- **Cons:** More expensive than DIY K8s

## Option 2: AWS EKS

- **EKS Cluster:** $0.10/hour control plane (~$73/month) + VM costs
- **Managed Services:** RDS PostgreSQL + ElastiCache Redis
- **Pros:** Mature ecosystem, lots of examples
- **Cons:** Complex pricing, many service options

## Option 3: Google GKE

- **GKE Cluster:** Free control plane (1 cluster) + VM costs
- **Managed Services:** Cloud SQL + Memorystore Redis
- **Pros:** Best GKE free tier, good for startups
- **Cons:** Smaller ecosystem than AWS/Azure

## Option 4: DigitalOcean Kubernetes

- **DOKS Cluster:** Free control plane + $40/month for 2 nodes (4GB each)
- **Managed Services:** Managed PostgreSQL + Managed Redis
- **Pros:** **Cheapest option**, simple pricing, great for labs
- **Cons:** Fewer features than big clouds

## Recommendation

**Start:** Docker Desktop K8s (local, free)
**Production (if needed):** DigitalOcean Kubernetes (~$80/month all-in)

All K8s manifests in this document work on any provider with minimal changes!

---

**Questions?** mail.thiagomendes@gmail.com
