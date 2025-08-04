# Service 模块文档

## 概述
Service模块是Orion Ops的业务逻辑层，负责处理核心业务功能。它提供了用户管理、认证授权、系统配置、任务调度等服务。

## 核心服务

### 1. 用户服务 (User Service)
文件: `user_service.rs`
- **UserService**: 用户管理服务，处理用户CRUD操作
- **User**: 用户实体，包含基本信息和权限
- **UserRole**: 用户角色枚举（Admin, Operator, Viewer）
- **UserStatus**: 用户状态（Active, Inactive, Locked）

### 2. 认证服务 (Auth Service)
文件: `auth_service.rs`
- **AuthService**: 认证授权服务，处理登录、权限验证
- **AuthToken**: 认证令牌，包含用户信息和权限
- **Permission**: 权限定义，基于RBAC模型
- **Session**: 用户会话管理

### 3. 系统配置服务 (System Config Service)
文件: `system_config_service.rs`
- **SystemConfigService**: 系统配置管理
- **Config**: 配置实体，支持多种配置类型
- **ConfigType**: 配置类型枚举（JSON, YAML, TOML）
- **ConfigScope**: 配置作用域（Global, User, Module）

### 4. 任务调度服务 (Task Scheduler Service)
文件: `task_scheduler_service.rs`
- **TaskSchedulerService**: 任务调度服务
- **ScheduledTask**: 定时任务定义
- **TaskStatus**: 任务状态（Pending, Running, Success, Failed）
- **TaskTrigger**: 任务触发器（Cron, Interval, Manual）

### 5. 审计服务 (Audit Service)
文件: `audit_service.rs`
- **AuditService**: 审计日志服务
- **AuditLog**: 审计日志实体
- **AuditLevel**: 审计级别（Info, Warning, Error, Critical）
- **AuditAction**: 审计动作类型

## 服务架构

### 服务层设计
```
┌─────────────────┐
│   Controller    │  ← HTTP/CLI接口
├─────────────────┤
│     Service     │  ← 业务逻辑层
```

### 服务架构图

```mermaid
graph TB
    subgraph "Service Layer Architecture"
        A[Controller Layer] --> B[Service Layer]
        B --> C[Repository Layer]
        
        subgraph "Core Services"
            B --> D[UserService]
            B --> E[AuthService]
            B --> F[SystemConfigService]
            B --> G[TaskSchedulerService]
            B --> H[AuditService]
            B --> I[PackageService]
            B --> J[WorkflowService]
        end
        
        subgraph "Service Components"
            D --> D1[User Entity]
            D --> D2[UserRole Enum]
            D --> D3[UserStatus Enum]
            
            E --> E1[AuthToken]
            E --> E2[Permission]
            E --> E3[Session]
            
            F --> F1[Config Entity]
            F --> F2[ConfigType Enum]
            F --> F3[ConfigScope Enum]
            
            G --> G1[ScheduledTask]
            G --> G2[TaskStatus Enum]
            G --> G3[TaskTrigger]
            
            H --> H1[AuditLog]
            H --> H2[AuditLevel Enum]
            H --> H3[AuditAction]
        end
        
        subgraph "Repository Layer"
            C --> K[UserRepository]
            C --> L[ConfigRepository]
            C --> M[TaskRepository]
            C --> N[AuditRepository]
            C --> O[PackageRepository]
            C --> P[WorkflowRepository]
        end
        
        subgraph "Cross-Cutting Concerns"
            Q[Transaction Manager] -.-> D
            Q -.-> E
            Q -.-> F
            Q -.-> G
            Q -.-> H
            
            R[Cache Manager] -.-> D
            R -.-> F
            R -.-> G
            
            S[Security Manager] -.-> E
            S -.-> H
            
            T[Logging Manager] -.-> D
            T -.-> E
            T -.-> F
            T -.-> G
            T -.-> H
        end
    end
    
    subgraph "Service Communication"
        D -.-> E
        E -.-> D
        F -.-> D
        G -.-> D
        H -.-> D
        H -.-> E
        H -.-> F
        H -.-> G
    end
    
    subgraph "External Dependencies"
        K --> DB[(Database)]
        L --> DB
        M --> DB
        N --> DB
        O --> DB
        P --> DB
        
        R --> Cache[(Redis Cache)]
        
        G --> Queue[(Message Queue)]
    end├─────────────────┤
│    Repository   │  ← 数据访问层
├─────────────────┤
│     Storage     │  ← 数据存储层
└─────────────────┘
```

### 服务注册与发现
```rust
use orion_ops::service::{ServiceRegistry, ServiceProvider};

let registry = ServiceRegistry::new();
registry.register("user-service", Box::new(UserService::new()));
registry.register("auth-service", Box::new(AuthService::new()));
```

## 核心功能

### 用户管理
```rust
use orion_ops::service::UserService;

let user_service = UserService::new();
let user = user_service.create_user("admin", "admin@example.com", UserRole::Admin)?;
let users = user_service.list_users(0, 10)?;
```

### 认证授权
```rust
use orion_ops::service::AuthService;

let auth_service = AuthService::new();
let token = auth_service.login("admin", "password")?;
let is_valid = auth_service.validate_token(&token)?;
let permissions = auth_service.get_user_permissions("admin")?;
```

### 系统配置
```rust
use orion_ops::service::SystemConfigService;

let config_service = SystemConfigService::new();
let config = config_service.get_config("database.url")?;
config_service.set_config("log.level", "debug", ConfigScope::Global)?;
```

### 任务调度
```rust
use orion_ops::service::TaskSchedulerService;

let scheduler = TaskSchedulerService::new();
let task = scheduler.create_task("backup", "0 0 * * *", "backup.sh")?;
scheduler.start_task(task.id)?;
```

## 数据结构

### User结构
```rust
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### AuthToken结构
```rust
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<Permission>,
}
```

### ScheduledTask结构
```rust
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub command: String,
    pub status: TaskStatus,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
}
```

## 错误处理

服务层提供了统一的错误处理：
- **ServiceError**: 服务层通用错误
- **AuthError**: 认证相关错误
- **ValidationError**: 数据验证错误

## 缓存策略

### 多级缓存
- **L1缓存**: 内存缓存，快速访问
- **L2缓存**: Redis缓存，分布式支持
- **L3缓存**: 数据库缓存，持久化存储

### 缓存失效
- TTL过期
- 主动失效
- 版本控制

## 监控与指标

### 服务指标
- 请求延迟
- 错误率
- 吞吐量
- 资源使用率

### 健康检查
```rust
pub trait HealthCheck {
    async fn health_check(&self) -> HealthStatus;
}
```

## 配置管理

### 服务配置
```yaml
service:
  user:
    max_users: 1000
    password_policy: strong
  auth:
    token_ttl: 3600
    max_sessions: 5
  scheduler:
    max_concurrent_tasks: 10
    retry_count: 3
```

## 扩展性

### 插件系统
- 自定义服务实现
- 服务钩子
- 事件监听

### 服务发现
- 基于配置的发现
- 基于DNS的发现
- 基于注册中心的发现

## 最佳实践

1. **服务隔离**: 每个服务独立部署，避免耦合
2. **接口设计**: 使用清晰的接口定义，避免实现细节泄露
3. **错误处理**: 提供有意义的错误信息，便于调试
4. **日志记录**: 记录关键操作和错误，便于审计
5. **性能优化**: 使用缓存和异步处理提高性能
6. **安全考虑**: 输入验证、权限检查、敏感信息脱敏