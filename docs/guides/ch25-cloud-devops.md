# Chapter 25: Cloud and DevOps

Fusion applications can be deployed to cloud environments using containerization, Kubernetes, CI/CD pipelines, and infrastructure as code. This chapter covers modern deployment practices.

## Containerization

### Dockerfile

```dockerfile
# Build stage
FROM fusion:2.0 AS builder

WORKDIR /app
COPY . .

# Build dependencies first for caching
RUN fusion build --release

# Runtime stage
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash appuser

COPY --from=builder /app/target/release/myapp /usr/local/bin/

USER appuser

EXPOSE 8080

CMD ["myapp"]
```

### Multi-stage Build

```dockerfile
# Frontend build stage
FROM node:18 AS frontend-builder

WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# Backend build stage
FROM fusion:2.0 AS backend-builder

WORKDIR /app
COPY backend/ .
RUN fusion build --release

# Final stage
FROM alpine:latest

RUN apk --no-cache add ca-certificates

WORKDIR /root/

COPY --from=backend-builder /app/target/release/server .
COPY --from=frontend-builder /app/dist ./static/

EXPOSE 8080

CMD ["./server"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/myapp
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
    networks:
      - app-network

  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=myapp
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - app-network

  redis:
    image: redis:7-alpine
    networks:
      - app-network

volumes:
  postgres_data:

networks:
  app-network:
    driver: bridge
```

## Kubernetes

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  labels:
    app: myapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
      - name: myapp
        image: myregistry/myapp:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: myapp-secrets
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
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
spec:
  selector:
    app: myapp
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

### Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: myapp-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - myapp.example.com
    secretName: myapp-tls
  rules:
  - host: myapp.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: myapp-service
            port:
              number: 80
```

### ConfigMap and Secret

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: myapp-config
data:
  RUST_LOG: "info"
  MAX_CONNECTIONS: "100"

---
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: myapp-secrets
type: Opaque
stringData:
  database-url: "postgres://user:password@host:5432/db"
  api-key: "your-api-key"
```

## CI/CD Pipelines

### GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Fusion
      run: |
        curl -sSf https://fusion-lang.org/install.sh | sh
        echo "$HOME/.fusion/bin" >> $GITHUB_PATH
    
    - name: Check formatting
      run: fusion fmt --check
    
    - name: Run linter
      run: fusion lint
    
    - name: Run tests
      run: fusion test
    
    - name: Build
      run: fusion build --release

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Fusion
      run: |
        curl -sSf https://fusion-lang.org/install.sh | sh
        echo "$HOME/.fusion/bin" >> $GITHUB_PATH
    
    - name: Build release
      run: fusion build --release
    
    - name: Build Docker image
      run: docker build -t myregistry/myapp:${{ github.sha }} .
    
    - name: Push to registry
      run: |
        echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
        docker push myregistry/myapp:${{ github.sha }}
    
    - name: Deploy to Kubernetes
      run: |
        kubectl set image deployment/myapp myapp=myregistry/myapp:${{ github.sha }}
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - test
  - build
  - deploy

variables:
  CARGO_TERM_COLOR: always

test:
  stage: test
  image: fusion:2.0
  script:
    - fusion fmt --check
    - fusion lint
    - fusion test
    - fusion build --release

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main

deploy:
  stage: deploy
  image: bitnami/kubectl
  script:
    - kubectl set image deployment/myapp myapp=$CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main
  when: manual
```

### Jenkins Pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    environment {
        FUSION_HOME = tool('Fusion')
        PATH = "${FUSION_HOME}/bin:${env.PATH}"
    }
    
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }
        
        stage('Test') {
            steps {
                sh 'fusion fmt --check'
                sh 'fusion lint'
                sh 'fusion test'
            }
        }
        
        stage('Build') {
            steps {
                sh 'fusion build --release'
            }
        }
        
        stage('Docker Build') {
            steps {
                script {
                    docker.build("myregistry/myapp:${env.BUILD_NUMBER}")
                }
            }
        }
        
        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                script {
                    docker.withRegistry('https://registry.example.com', 'credentials-id') {
                        docker.image("myregistry/myapp:${env.BUILD_NUMBER}").push()
                    }
                }
                sh "kubectl set image deployment/myapp myapp=myregistry/myapp:${env.BUILD_NUMBER}"
            }
        }
    }
    
    post {
        always {
            cleanWs()
        }
    }
}
```

## Infrastructure as Code

### Terraform

```hcl
# main.tf
provider "aws" {
  region = "us-east-1"
}

# ECS Cluster
resource "aws_ecs_cluster" "app" {
  name = "myapp-cluster"
}

# Task Definition
resource "aws_ecs_task_definition" "app" {
  family                   = "myapp"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = 256
  memory                   = 512
  
  container_definitions = jsonencode([{
    name  = "myapp"
    image = "myregistry/myapp:latest"
    
    portMappings = [{
      containerPort = 8080
      hostPort      = 8080
    }]
    
    environment = [
      {
        name  = "DATABASE_URL"
        value = aws_db_instance.postgres.endpoint
      }
    ]
    
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.app.name
        "awslogs-region"        = "us-east-1"
        "awslogs-stream-prefix" = "ecs"
      }
    }
  }])
}

# ECS Service
resource "aws_ecs_service" "app" {
  name            = "myapp"
  cluster         = aws_ecs_cluster.app.id
  task_definition = aws_ecs_task_definition.app.arn
  desired_count   = 2
  
  load_balancer {
    target_group_arn = aws_lb_target_group.app.arn
    container_name   = "myapp"
    container_port   = 8080
  }
}

# RDS PostgreSQL
resource "aws_db_instance" "postgres" {
  identifier     = "myapp-db"
  engine         = "postgres"
  engine_version = "15"
  instance_class = "db.t3.micro"
  
  allocated_storage     = 20
  max_allocated_storage = 100
  
  db_name  = "myapp"
  username = "admin"
  password = var.db_password
  
  skip_final_snapshot = true
}

# ALB
resource "aws_lb" "app" {
  name               = "myapp-lb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.lb.id]
  subnets            = var.public_subnets
}

resource "aws_lb_target_group" "app" {
  name        = "myapp-tg"
  port        = 8080
  protocol    = "HTTP"
  vpc_id      = var.vpc_id
  
  health_check {
    path = "/health"
  }
}

resource "aws_security_group" "lb" {
  name        = "myapp-lb-sg"
  description = "Security group for ALB"
  
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
```

### Pulumi

```typescript
// index.ts
import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";

// Create ECS cluster
const cluster = new aws.ecs.Cluster("app-cluster");

// Create task definition
const taskDefinition = new aws.ecs.TaskDefinition("app-task", {
    family: "myapp",
    networkMode: "awsvpc",
    requiresCompatibilities: ["FARGATE"],
    cpu: "256",
    memory: "512",
    containerDefinitions: JSON.stringify([{
        name: "myapp",
        image: "myregistry/myapp:latest",
        portMappings: [{
            containerPort: 8080,
            hostPort: 8080
        }],
        environment: [{
            name: "DATABASE_URL",
            value: postgres.endpoint
        }]
    }])
});

// Create ECS service
const service = new aws.ecs.Service("app-service", {
    cluster: cluster.arn,
    taskDefinition: taskDefinition.arn,
    desiredCount: 2,
    launchType: "FARGATE",
    networkConfiguration: {
        subnets: publicSubnets,
        securityGroups: [appSecurityGroup.id]
    }
});

// Create RDS instance
const postgres = new aws.rds.Instance("app-db", {
    identifier: "myapp-db",
    engine: "postgres",
    engineVersion: "15",
    instanceClass: "db.t3.micro",
    allocatedStorage: 20,
    dbName: "myapp",
    username: "admin",
    password: dbPassword,
    skipFinalSnapshot: true
});
```

## Monitoring and Logging

### Prometheus Metrics

```fusion
use prometheus::{Encoder, TextEncoder, Registry, Counter, Histogram, Gauge};

struct Metrics {
    registry: Registry,
    request_counter: Counter,
    request_duration: Histogram,
    active_connections: Gauge,
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();
        
        let request_counter = Counter::new(
            "http_requests_total",
            "Total number of HTTP requests"
        ).unwrap();
        
        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds"
            )
        ).unwrap();
        
        let active_connections = Gauge::new(
            "active_connections",
            "Number of active connections"
        ).unwrap();
        
        registry.register(Box::new(request_counter.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();
        
        Self {
            registry,
            request_counter,
            request_duration,
            active_connections,
        }
    }
    
    fn record_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        self.request_counter.with_label_values(&[method, path, &status.to_string()]).inc();
        self.request_duration.observe(duration);
    }
    
    fn metrics_handler(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}
```

### Structured Logging

```fusion
use tracing::{info, warn, error, debug, span, Level};
use tracing_subscriber::EnvFilter;

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

fn handle_request(request: &Request) {
    let span = span!(
        Level::INFO,
        "request",
        method = %request.method,
        path = %request.path,
        id = %request.id,
    );
    
    let _enter = span.enter();
    
    info!("Request received");
    
    match process_request(request) {
        Ok(response) => {
            info!(status = response.status, "Request completed");
        }
        Err(e) => {
            error!(error = %e, "Request failed");
        }
    }
}
```

## Summary

Fusion's cloud and DevOps capabilities include:

1. **Containerization**: Docker multi-stage builds and Docker Compose
2. **Kubernetes**: Deployments, services, ingress, and config management
3. **CI/CD Pipelines**: GitHub Actions, GitLab CI, and Jenkins
4. **Infrastructure as Code**: Terraform and Pulumi for cloud resources
5. **Monitoring and Logging**: Prometheus metrics and structured logging

Fusion's performance and safety make it ideal for cloud-native applications, while the ecosystem provides all the tools needed for modern deployment workflows.

In the next chapter, we'll explore blockchain development with Fusion.