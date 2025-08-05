# Deployment Guide

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Database Configuration](#database-configuration)
4. [Application Deployment](#application-deployment)
5. [Infrastructure as Code](#infrastructure-as-code)
6. [CI/CD Pipeline](#cicd-pipeline)
7. [Monitoring and Logging](#monitoring-and-logging)
8. [Security Considerations](#security-considerations)
9. [Troubleshooting](#troubleshooting)
10. [Rollback Procedures](#rollback-procedures)

## Prerequisites

### System Requirements

- **Operating System**: Ubuntu 20.04 LTS or later
- **CPU**: Minimum 4 cores, recommended 8 cores
- **RAM**: Minimum 8GB, recommended 16GB
- **Storage**: 100GB SSD minimum
- **Network**: 1Gbps connection

### Software Dependencies

```bash
# Required software versions
- Docker: 20.10+
- Docker Compose: 2.0+
- Kubernetes: 1.25+
- Helm: 3.10+
- Node.js: 18.x LTS
- PostgreSQL: 14+
- Redis: 7.0+
- Nginx: 1.22+
```

### Access Requirements

- AWS/GCP/Azure account with appropriate permissions
- Docker Hub account for image registry
- SSL certificates for HTTPS
- Domain name configured with DNS

## Environment Setup

### Development Environment

```bash
# Clone the repository
git clone https://github.com/example/ecommerce-platform.git
cd ecommerce-platform

# Copy environment template
cp .env.example .env.development

# Install dependencies
npm install
pip install -r requirements.txt
bundle install

# Start development services
docker-compose -f docker-compose.dev.yml up -d
```

### Staging Environment

```yaml
# docker-compose.staging.yml
version: '3.8'

services:
  app:
    image: ecommerce/app:staging
    environment:
      - NODE_ENV=staging
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
    ports:
      - "3000:3000"
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:14-alpine
    environment:
      - POSTGRES_DB=ecommerce_staging
      - POSTGRES_USER=${DB_USER}
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"

  nginx:
    image: nginx:alpine
    volumes:
      - ./nginx/staging.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - app

volumes:
  postgres_data:
  redis_data:
```

### Production Environment

```bash
# Production environment variables
export NODE_ENV=production
export DATABASE_URL=postgresql://user:pass@db.example.com:5432/ecommerce_prod
export REDIS_URL=redis://redis.example.com:6379
export AWS_REGION=us-east-1
export S3_BUCKET=ecommerce-assets-prod
```

## Database Configuration

### Initial Setup

```sql
-- Create production database
CREATE DATABASE ecommerce_production;
CREATE USER ecommerce_user WITH ENCRYPTED PASSWORD 'strong_password';
GRANT ALL PRIVILEGES ON DATABASE ecommerce_production TO ecommerce_user;

-- Enable required extensions
\c ecommerce_production
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
CREATE EXTENSION IF NOT EXISTS "btree_gin";
```

### Migration Strategy

```bash
# Run migrations
npm run migrate:up

# Verify migration status
npm run migrate:status

# Rollback if needed
npm run migrate:down
```

### Backup Configuration

```bash
#!/bin/bash
# backup.sh - Daily backup script

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/postgres"
DB_NAME="ecommerce_production"

# Create backup
pg_dump -h localhost -U ecommerce_user -d $DB_NAME -F custom -f "$BACKUP_DIR/backup_$DATE.dump"

# Upload to S3
aws s3 cp "$BACKUP_DIR/backup_$DATE.dump" s3://ecommerce-backups/postgres/

# Clean up old backups (keep last 30 days)
find $BACKUP_DIR -name "backup_*.dump" -mtime +30 -delete
```

## Application Deployment

### Docker Build Process

```dockerfile
# Dockerfile.production
FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY yarn.lock ./

# Install dependencies
RUN yarn install --frozen-lockfile --production=false

# Copy source code
COPY . .

# Build application
RUN yarn build

# Production stage
FROM node:18-alpine

WORKDIR /app

# Install production dependencies only
COPY package*.json ./
RUN yarn install --frozen-lockfile --production=true

# Copy built application
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/public ./public

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001

# Change ownership
RUN chown -R nodejs:nodejs /app

USER nodejs

EXPOSE 3000

CMD ["node", "dist/server.js"]
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ecommerce-app
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ecommerce
  template:
    metadata:
      labels:
        app: ecommerce
    spec:
      containers:
      - name: app
        image: ecommerce/app:v1.0.0
        ports:
        - containerPort: 3000
        env:
        - name: NODE_ENV
          value: "production"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: ecommerce-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: ecommerce-service
  namespace: production
spec:
  selector:
    app: ecommerce
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

### Helm Chart

```yaml
# helm/values.yaml
replicaCount: 3

image:
  repository: ecommerce/app
  tag: v1.0.0
  pullPolicy: IfNotPresent

service:
  type: LoadBalancer
  port: 80

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: api-tls
      hosts:
        - api.example.com

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

postgresql:
  enabled: true
  auth:
    database: ecommerce
    username: ecommerce_user
  persistence:
    enabled: true
    size: 50Gi

redis:
  enabled: true
  auth:
    enabled: true
  persistence:
    enabled: true
    size: 10Gi
```

## Infrastructure as Code

### Terraform Configuration

```hcl
# terraform/main.tf
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
  }
  
  backend "s3" {
    bucket = "ecommerce-terraform-state"
    key    = "production/terraform.tfstate"
    region = "us-east-1"
  }
}

# VPC Configuration
module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
  
  name = "ecommerce-vpc"
  cidr = "10.0.0.0/16"
  
  azs             = ["us-east-1a", "us-east-1b", "us-east-1c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
  
  enable_nat_gateway = true
  enable_vpn_gateway = true
  
  tags = {
    Environment = "production"
    Project     = "ecommerce"
  }
}

# EKS Cluster
module "eks" {
  source = "terraform-aws-modules/eks/aws"
  
  cluster_name    = "ecommerce-cluster"
  cluster_version = "1.25"
  
  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets
  
  eks_managed_node_groups = {
    main = {
      desired_capacity = 3
      max_capacity     = 10
      min_capacity     = 3
      
      instance_types = ["t3.medium"]
      
      k8s_labels = {
        Environment = "production"
        Application = "ecommerce"
      }
    }
  }
}

# RDS Database
resource "aws_db_instance" "postgres" {
  identifier = "ecommerce-postgres"
  
  engine         = "postgres"
  engine_version = "14.7"
  instance_class = "db.r5.large"
  
  allocated_storage     = 100
  max_allocated_storage = 1000
  storage_encrypted     = true
  
  db_name  = "ecommerce_production"
  username = "ecommerce_admin"
  password = var.db_password
  
  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.main.name
  
  backup_retention_period = 30
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  enabled_cloudwatch_logs_exports = ["postgresql"]
  
  tags = {
    Environment = "production"
    Project     = "ecommerce"
  }
}
```

## CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]
  workflow_dispatch:

env:
  REGISTRY: docker.io
  IMAGE_NAME: ecommerce/app

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run tests
        run: npm test
      
      - name: Run security audit
        run: npm audit
      
      - name: SonarQube Scan
        uses: sonarsource/sonarqube-scan-action@master
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}

  build:
    needs: test
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.meta.outputs.version }}
    steps:
      - uses: actions/checkout@v3
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2
      
      - name: Log in to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}
      
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=sha,prefix={{branch}}-
            type=semver,pattern={{version}}
      
      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          file: ./Dockerfile.production
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=registry,ref=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:buildcache
          cache-to: type=registry,ref=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:buildcache,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v3
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1
      
      - name: Update kubeconfig
        run: |
          aws eks update-kubeconfig --name ecommerce-cluster --region us-east-1
      
      - name: Deploy to Kubernetes
        run: |
          helm upgrade --install ecommerce ./helm \
            --namespace production \
            --set image.tag=${{ needs.build.outputs.version }} \
            --wait \
            --timeout 10m
      
      - name: Verify deployment
        run: |
          kubectl rollout status deployment/ecommerce-app -n production
          kubectl get pods -n production
```

## Monitoring and Logging

### Prometheus Configuration

```yaml
# prometheus/values.yaml
prometheus:
  prometheusSpec:
    serviceMonitorSelectorNilUsesHelmValues: false
    retention: 30d
    storageSpec:
      volumeClaimTemplate:
        spec:
          storageClassName: gp2
          accessModes: ["ReadWriteOnce"]
          resources:
            requests:
              storage: 100Gi
    
    additionalScrapeConfigs:
      - job_name: 'ecommerce-app'
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names: ['production']
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_label_app]
            action: keep
            regex: ecommerce
```

### Grafana Dashboards

```json
{
  "dashboard": {
    "title": "E-commerce Application Metrics",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m])) by (status)"
          }
        ]
      },
      {
        "title": "Response Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m]))"
          }
        ]
      }
    ]
  }
}
```

### ELK Stack Configuration

```yaml
# elasticsearch/values.yaml
elasticsearch:
  replicas: 3
  resources:
    requests:
      cpu: "1000m"
      memory: "2Gi"
    limits:
      cpu: "2000m"
      memory: "4Gi"
  
  volumeClaimTemplate:
    accessModes: ["ReadWriteOnce"]
    resources:
      requests:
        storage: 100Gi

# Logstash pipeline
input {
  beats {
    port => 5044
  }
}

filter {
  if [kubernetes][container][name] == "ecommerce-app" {
    grok {
      match => {
        "message" => "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{GREEDYDATA:message}"
      }
    }
    
    date {
      match => ["timestamp", "ISO8601"]
    }
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
    index => "ecommerce-%{+YYYY.MM.dd}"
  }
}
```

## Security Considerations

### SSL/TLS Configuration

```nginx
# nginx/ssl.conf
server {
    listen 443 ssl http2;
    server_name api.example.com;
    
    ssl_certificate /etc/nginx/ssl/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    location / {
        proxy_pass http://ecommerce-app:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Security Scanning

```bash
#!/bin/bash
# security-scan.sh

# Scan Docker images for vulnerabilities
trivy image ecommerce/app:latest

# Scan Kubernetes manifests
kubesec scan k8s/*.yaml

# Check for secrets in code
trufflehog --regex --entropy=False .

# OWASP dependency check
dependency-check --project "E-commerce" --scan . --format JSON
```

## Troubleshooting

### Common Issues

#### Database Connection Issues

```bash
# Check database connectivity
kubectl exec -it deployment/ecommerce-app -n production -- nc -zv postgres-service 5432

# View database logs
kubectl logs -n production deployment/postgres --tail=100

# Test connection with psql
kubectl run -it --rm debug --image=postgres:14 --restart=Never -- \
  psql -h postgres-service -U ecommerce_user -d ecommerce_production
```

#### Application Crashes

```bash
# Check pod status
kubectl get pods -n production
kubectl describe pod <pod-name> -n production

# View application logs
kubectl logs -n production deployment/ecommerce-app --tail=100 -f

# Check resource usage
kubectl top pods -n production
kubectl top nodes
```

#### Performance Issues

```bash
# Generate heap dump
kubectl exec -it <pod-name> -n production -- kill -USR1 1
kubectl cp <pod-name>:/tmp/heapdump.hprof ./heapdump.hprof -n production

# Profile CPU usage
kubectl exec -it <pod-name> -n production -- node --prof app.js
```

### Debug Mode

```yaml
# Enable debug mode in deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ecommerce-app-debug
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: NODE_ENV
          value: "development"
        - name: DEBUG
          value: "*"
        - name: LOG_LEVEL
          value: "debug"
```

## Rollback Procedures

### Kubernetes Rollback

```bash
# View deployment history
kubectl rollout history deployment/ecommerce-app -n production

# Rollback to previous version
kubectl rollout undo deployment/ecommerce-app -n production

# Rollback to specific revision
kubectl rollout undo deployment/ecommerce-app -n production --to-revision=2

# Monitor rollback status
kubectl rollout status deployment/ecommerce-app -n production
```

### Database Rollback

```bash
#!/bin/bash
# db-rollback.sh

BACKUP_FILE=$1
DB_NAME="ecommerce_production"

# Stop application
kubectl scale deployment/ecommerce-app -n production --replicas=0

# Restore database
pg_restore -h localhost -U ecommerce_user -d $DB_NAME -c $BACKUP_FILE

# Start application
kubectl scale deployment/ecommerce-app -n production --replicas=3
```

### Blue-Green Deployment

```yaml
# blue-green-switch.yaml
apiVersion: v1
kind: Service
metadata:
  name: ecommerce-service
  namespace: production
spec:
  selector:
    app: ecommerce
    version: green  # Switch between blue/green
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
```

## Post-Deployment Checklist

- [ ] Verify all pods are running
- [ ] Check application health endpoints
- [ ] Validate database connections
- [ ] Test critical user flows
- [ ] Verify monitoring dashboards
- [ ] Check log aggregation
- [ ] Run smoke tests
- [ ] Verify SSL certificates
- [ ] Test auto-scaling
- [ ] Check backup procedures
- [ ] Update documentation
- [ ] Notify stakeholders

## Maintenance Windows

```bash
# Schedule maintenance
kubectl annotate deployment ecommerce-app -n production \
  maintenance.start="2024-01-20T02:00:00Z" \
  maintenance.end="2024-01-20T04:00:00Z" \
  maintenance.reason="Database upgrade"
```

## Support Contacts

- **DevOps Team**: devops@example.com
- **On-Call Engineer**: +1-555-0123 (24/7)
- **Escalation**: engineering-lead@example.com
- **Incident Response**: https://incident.example.com