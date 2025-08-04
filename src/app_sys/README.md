# App System 模块文档

## 概述
App System模块是Orion Ops的应用系统管理核心，负责系统级项目的创建、配置、部署和生命周期管理。它提供了完整的应用系统抽象，支持多环境部署、配置管理、服务编排和系统监控功能。

## 系统架构

### 应用系统设计
```
┌─────────────────────┐
│   AppSystem         │  ← 应用系统
├─────────────────────┤
│   SysProject        │  ← 系统项目
├─────────────────────┤
│   Environment       │  ← 运行环境
├─────────────────────┤
│   Deployment        │  ← 部署配置
└─────────────────────┘
```

### 应用系统架构图

```mermaid
graph TB
    subgraph "App System Core"
        A[AppSystem] --> B[SysProject]
        A --> C[EnvironmentManager]
        A --> D[ServiceOrchestrator]
        A --> E[DeploymentManager]
        A --> F[ConfigurationManager]
        A --> G[MonitoringManager]
        
        B --> H[ProjectConfig]
        B --> I[Service]
        B --> J[Environment]
        B --> K[Deployment]
        B --> L[MonitoringConfig]
        
        C --> M[DevelopmentEnv]
        C --> N[TestingEnv]
        C --> O[StagingEnv]
        C --> P[ProductionEnv]
        
        D --> Q[ContainerService]
        D --> R[VirtualMachineService]
        D --> S[ServerlessService]
        
        E --> T[DeploymentPipeline]
        E --> U[RollbackManager]
        E --> V[HealthChecker]
        
        F --> W[ConfigStore]
        F --> X[ConfigValidator]
        F --> Y[ConfigWatcher]
        
        G --> Z[MetricsCollector]
        G --> AA[AlertManager]
        G --> AB[LogAggregator]
    end
    
    subgraph "Environment Types"
        M --> M1[Local Development]
        M --> M2[Docker Compose]
        N --> N1[Unit Testing]
        N --> N2[Integration Testing]
        O --> O1[Pre-production]
        O --> O2[Performance Testing]
        P --> P1[High Availability]
        P --> P2[Load Balanced]
    end
    
    subgraph "Service Components"
        Q --> Q1[Docker Container]
        Q --> Q2[Kubernetes Pod]
        Q --> Q3[Docker Compose]
        
        R --> R1[VM Instance]
        R --> R2[Cloud Instance]
        R --> R3[Bare Metal]
        
        S --> S1[Function]
        S --> S2[Lambda]
        S --> S3[Cloud Function]
    end
    
    subgraph "Deployment Stages"
        T --> T1[Build Stage]
        T --> T2[Test Stage]
        T --> T3[Deploy Stage]
        T --> T4[Verify Stage]
        
        U --> U1[Rollback Strategy]
        U --> U2[Backup Restore]
        U --> U3[Blue-Green Deploy]
    end
    
    subgraph "Configuration Sources"
        W --> W1[Environment Variables]
        W --> W2[Config Files]
        W --> W3[Database Config]
        W --> W4[Remote Config Service]
    end
    
    subgraph "Monitoring Stack"
        Z --> Z1[System Metrics]
        Z --> Z2[Application Metrics]
        Z --> Z3[Business Metrics]
        
        AA --> AA1[Email Alerts]
        AA --> AA2[Slack Notifications]
        AA --> AA3[PagerDuty Integration]
        
        AB --> AB1[Log Collection]
        AB --> AB2[Log Analysis]
        AB --> AB3[Log Storage]
    end
    
    subgraph "External Integrations"
        A --> AC[CI/CD Pipeline]
        A --> AD[Cloud Providers]
        A --> AE[Container Registry]
        A --> AF[Monitoring Services]
    end

### 核心职责
- **系统项目管理**: 创建和管理系统级项目
- **环境配置**: 多环境配置管理（开发、测试、生产）
- **服务编排**: 容器化服务的编排和管理
- **部署管理**: 自动化部署和回滚
- **监控告警**: 系统健康监控和告警
- **配置管理**: 集中化配置管理

## 核心组件

### 1. 系统项目 (SysProject)
文件: `sysproj.rs`
- **SysProject**: 系统项目结构定义
- **ProjectConfig**: 项目配置管理
- **Environment**: 运行环境定义
- **Service**: 服务定义和配置
- **Deployment**: 部署配置

#### 系统项目结构
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysProject {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub environments: Vec<Environment>,
    pub services: Vec<Service>,
    pub configurations: HashMap<String, ConfigValue>,
    pub deployments: Vec<Deployment>,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub description: Option<String>,
    pub variables: HashMap<String, String>,
    pub resources: ResourceLimits,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub image: String,
    pub tag: String,
    pub ports: Vec<PortMapping>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    pub health_check: HealthCheck,
    pub replicas: u32,
    pub resources: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub name: String,
    pub environment: String,
    pub strategy: DeploymentStrategy,
    pub rollback: RollbackConfig,
    pub health_check: HealthCheck,
    pub timeout: Duration,
}
```

### 2. 应用系统管理器 (AppSystem)
文件: `mod.rs`
- **AppSystem**: 应用系统管理器主类
- **ProjectManager**: 项目管理器
- **EnvironmentManager**: 环境管理器
- **ServiceManager**: 服务管理器
- **DeploymentManager**: 部署管理器
- **ConfigManager**: 配置管理器

#### 应用系统管理器功能
```rust
use galaxy_ops::app_sys::{AppSystem, SysProject};

let system = AppSystem::new();

// 创建系统项目
let project = SysProject::new("my-app")
    .version("1.0.0")
    .description("My application system")
    .build();

system.create_project(project).await?;

// 部署项目
system.deploy("my-app", "production").await?;

// 获取项目状态
let status = system.get_status("my-app").await?;

// 更新配置
system.update_config("my-app", "database.url", "postgres://...").await?;

// 回滚部署
system.rollback("my-app", "production", "v1.0.0").await?;
```

## 环境管理

### 环境配置
```yaml
# environments.yml
environments:
  development:
    description: "Development environment"
    variables:
      LOG_LEVEL: "debug"
      DATABASE_URL: "postgres://dev:dev@localhost:5432/dev"
      REDIS_URL: "redis://localhost:6379"
    resources:
      cpu: "500m"
      memory: "512Mi"
    network:
      type: "bridge"
      ports:
        - "8080:8080"
    storage:
      type: "local"
      size: "1Gi"

  staging:
    description: "Staging environment"
    variables:
      LOG_LEVEL: "info"
      DATABASE_URL: "postgres://staging:staging@staging-db:5432/staging"
      REDIS_URL: "redis://staging-redis:6379"
    resources:
      cpu: "1000m"
      memory: "1Gi"
    network:
      type: "overlay"
      ports:
        - "80:8080"
    storage:
      type: "nfs"
      size: "10Gi"

  production:
    description: "Production environment"
    variables:
      LOG_LEVEL: "warn"
      DATABASE_URL: "postgres://prod:prod@prod-db:5432/prod"
      REDIS_URL: "redis://prod-redis:6379"
    resources:
      cpu: "2000m"
      memory: "2Gi"
    network:
      type: "overlay"
      ports:
        - "443:8080"
    storage:
      type: "persistent"
      size: "100Gi"
```

### 环境变量管理
```rust
use galaxy_ops::app_sys::{EnvironmentManager, ConfigValue};

let env_manager = EnvironmentManager::new();

// 设置环境变量
env_manager.set_variable("production", "API_KEY", "secret123").await?;

// 获取环境变量
let api_key = env_manager.get_variable("production", "API_KEY").await?;

// 批量设置变量
let variables = vec![
    ("DATABASE_URL", "postgres://..."),
    ("REDIS_URL", "redis://..."),
    ("LOG_LEVEL", "info"),
];
env_manager.set_variables("production", variables).await?;

// 环境变量模板
let template = env_manager.get_template("production").await?;
```

## 服务编排

### 服务定义
```yaml
# services.yml
services:
  web:
    image: "nginx:alpine"
    tag: "latest"
    ports:
      - "80:80"
      - "443:443"
    environment:
      NGINX_HOST: "localhost"
      NGINX_PORT: "80"
    volumes:
      - "/var/www:/usr/share/nginx/html"
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost/health"]
      interval: "30s"
      timeout: "10s"
      retries: 3
    replicas: 2
    resources:
      limits:
        cpu: "500m"
        memory: "512Mi"
      requests:
        cpu: "250m"
        memory: "256Mi"

  api:
    image: "myapp/api"
    tag: "v1.2.3"
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: "${DATABASE_URL}"
      REDIS_URL: "${REDIS_URL}"
      LOG_LEVEL: "${LOG_LEVEL}"
    volumes:
      - "/var/log/api:/app/logs"
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: "30s"
      timeout: "10s"
      retries: 3
    replicas: 3
    resources:
      limits:
        cpu: "1000m"
        memory: "1Gi"
      requests:
        cpu: "500m"
        memory: "512Mi"

  database:
    image: "postgres:13"
    tag: "alpine"
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: "myapp"
      POSTGRES_USER: "myuser"
      POSTGRES_PASSWORD: "mypassword"
    volumes:
      - "postgres_data:/var/lib/postgresql/data"
    health_check:
      test: ["CMD-SHELL", "pg_isready -U myuser -d myapp"]
      interval: "30s"
      timeout: "10s"
      retries: 3
    replicas: 1
    resources:
      limits:
        cpu: "1000m"
        memory: "2Gi"
      requests:
        cpu: "500m"
        memory: "1Gi"
```

### 服务依赖管理
```rust
use galaxy_ops::app_sys::{ServiceManager, ServiceDependency};

let service_manager = ServiceManager::new();

// 定义服务依赖
let dependencies = vec![
    ServiceDependency::new("api", "database"),
    ServiceDependency::new("web", "api"),
];

service_manager.set_dependencies(dependencies).await?;

// 按依赖顺序启动服务
service_manager.start_with_dependencies("production").await?;

// 检查服务健康状态
let health = service_manager.check_health("production").await?;
```

## 部署管理

### 部署策略
```yaml
# deployments.yml
deployments:
  rolling_update:
    name: "rolling-update"
    strategy: "rolling"
    max_unavailable: 1
    max_surge: 1
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: "30s"
      timeout: "10s"
      retries: 3
    timeout: "10m"
    rollback:
      enabled: true
      revision: "previous"

  blue_green:
    name: "blue-green"
    strategy: "blue-green"
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: "30s"
      timeout: "10s"
      retries: 3
    timeout: "15m"
    rollback:
      enabled: true
      revision: "blue"

  canary:
    name: "canary"
    strategy: "canary"
    weight: 10
    increment: 10
    interval: "5m"
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: "30s"
      timeout: "10s"
      retries: 3
    timeout: "30m"
    rollback:
      enabled: true
      revision: "stable"
```

### 部署流程
```rust
use galaxy_ops::app_sys::{DeploymentManager, DeploymentStrategy};

let deployment_manager = DeploymentManager::new();

// 创建部署
let deployment = deployment_manager
    .create_deployment("my-app", "production")
    .strategy(DeploymentStrategy::RollingUpdate)
    .build();

// 执行部署
let result = deployment_manager.deploy(deployment).await?;

// 监控部署状态
let status = deployment_manager.get_deployment_status("my-app", "production").await?;

// 回滚部署
deployment_manager.rollback("my-app", "production", "v1.0.0").await?;
```

## 配置管理

### 配置层次
```yaml
# config.yml
config:
  global:
    log_level: "info"
    timezone: "UTC"
    
  environment:
    development:
      debug: true
      log_level: "debug"
    
    staging:
      debug: false
      log_level: "info"
    
    production:
      debug: false
      log_level: "warn"
  
  service:
    api:
      port: 8080
      workers: 4
    
    web:
      port: 80
      workers: 2
  
  database:
    pool_size: 10
    timeout: 30
```

### 配置热更新
```rust
use galaxy_ops::app_sys::{ConfigManager, ConfigWatcher};

let config_manager = ConfigManager::new();

// 加载配置
let config = config_manager.load_config("my-app").await?;

// 监听配置变化
let watcher = ConfigWatcher::new("my-app");
watcher.on_change(|key, old_value, new_value| {
    println!("配置变更: {} = {} -> {}", key, old_value, new_value);
    
    // 应用配置变更
    apply_config_change(key, new_value);
});

// 动态更新配置
config_manager.update_config("my-app", "api.port", "9090").await?;
```

## 监控和告警

### 监控指标
```rust
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkMetrics,
    pub service_health: HashMap<String, ServiceHealth>,
    pub deployment_status: DeploymentStatus,
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub response_time: Duration,
    pub error_rate: f64,
    pub throughput: u64,
}
```

### 告警配置
```yaml
# monitoring.yml
monitoring:
  metrics:
    collection_interval: "30s"
    retention_period: "7d"
  
  alerts:
    - name: "high_cpu"
      condition: "cpu_usage > 80"
      severity: "warning"
      channels: ["email", "slack"]
    
    - name: "service_down"
      condition: "service_health.status == 'down'"
      severity: "critical"
      channels: ["email", "slack", "pagerduty"]
    
    - name: "deployment_failed"
      condition: "deployment_status == 'failed'"
      severity: "critical"
      channels: ["email", "slack"]
  
  channels:
    email:
      smtp: "smtp.company.com"
      recipients: ["ops@company.com"]
    
    slack:
      webhook: "https://hooks.slack.com/services/..."
      channel: "#alerts"
    
    pagerduty:
      service_key: "${PAGERDUTY_KEY}"
```

### 监控集成
```rust
use galaxy_ops::app_sys::{MonitoringManager, AlertManager};

let monitoring = MonitoringManager::new();

// 收集系统指标
let metrics = monitoring.collect_metrics("my-app").await?;

// 设置告警规则
let alert_manager = AlertManager::new();
alert_manager.add_rule("high_memory", "memory_usage > 90").await?;

// 发送告警
alert_manager.send_alert("high_memory", "Memory usage is 95%").await?;
```

## 日志管理

### 日志配置
```yaml
# logging.yml
logging:
  level: "info"
  format: "json"
  output: "stdout"
  
  rotation:
    enabled: true
    max_size: "100MB"
    max_files: 10
    max_age: "7d"
  
  aggregation:
    enabled: true
    endpoint: "http://log-aggregator:8080"
    buffer_size: 1000
```

### 日志收集
```rust
use galaxy_ops::app_sys::{LogManager, LogCollector};

let log_manager = LogManager::new();

// 收集服务日志
let logs = log_manager.collect_logs("my-app", "production").await?;

// 搜索日志
let results = log_manager.search_logs("my-app", "ERROR").await?;

// 日志分析
let analysis = log_manager.analyze_logs("my-app", Duration::from_hours(1)).await?;
```

## 备份和恢复

### 备份策略
```yaml
# backup.yml
backup:
  schedule: "0 2 * * *"  # 每天凌晨2点
  retention: "30d"
  compression: true
  encryption: true
  
  targets:
    - type: "database"
      name: "postgres"
      connection: "${DATABASE_URL}"
    
    - type: "files"
      name: "config"
      path: "/etc/myapp"
    
    - type: "volumes"
      name: "data"
      volumes: ["postgres_data