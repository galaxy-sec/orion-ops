# Storage 模块文档

## 概述
Storage模块是Orion Ops的数据存储层，负责数据的持久化存储和管理。它提供了统一的数据访问接口，支持多种存储后端，包括关系型数据库、键值存储、文件系统等。

## 存储架构

### 存储层设计
```
┌─────────────────┐
│   Repository    │  ← 数据访问接口
├─────────────────┤
│   Storage       │  ← 存储实现层
├─────────────────┤
│   Backend       │  ← 存储后端
└─────────────────┘
```

### 存储架构图

```mermaid
graph TB
    subgraph "Storage Layer Architecture"
        A[Repository Layer] --> B[Storage Layer]
        B --> C[Backend Layer]
        
        subgraph "Repository Components"
            A --> D[UserRepository]
            A --> E[ConfigRepository]
            A --> F[AuditRepository]
            A --> G[TaskRepository]
            A --> H[PackageRepository]
            A --> I[WorkflowRepository]
            A --> J[ProjectRepository]
        end
        
        subgraph "Storage Implementations"
            B --> K[DatabaseStorage]
            B --> L[CacheStorage]
            B --> M[FileStorage]
            B --> N[ObjectStorage]
        end
        
        subgraph "Database Backends"
            K --> O[PostgreSQL]
            K --> P[SQLite]
            K --> Q[MySQL]
        end
        
        subgraph "Cache Backends"
            L --> R[Redis]
            L --> S[MemoryCache]
            L --> T[Memcached]
        end
        
        subgraph "File Backends"
            M --> U[LocalFileSystem]
            M --> V[NetworkFileSystem]
            M --> W[EncryptedFileSystem]
        end
        
        subgraph "Object Storage Backends"
            N --> X[MinIO]
            N --> Y[AWS S3]
            N --> Z[Azure Blob]
        end
        
        subgraph "Storage Management"
            AA[DatabaseManager] --> K
            AB[CacheManager] --> L
            AC[FileStorageManager] --> M
            AD[ObjectStorageManager] --> N
        end
        
        subgraph "Connection Management"
            AA --> AE[ConnectionPool]
            AA --> AF[ConnectionConfig]
            AA --> AG[ConnectionMonitor]
        end
        
        subgraph "Cache Strategies"
            AB --> AH[LRU Cache]
            AB --> AI[LFU Cache]
            AB --> AJ[TTL Cache]
        end
    end
    
    subgraph "Data Flow"
        D -.-> K
        D -.-> L
        E -.-> K
        F -.-> K
        G -.-> K
        G -.-> L
        H -.-> K
        H -.-> N
        I -.-> K
        I -.-> N
        J -.-> K
        J -.-> M
    end
    
    subgraph "Cross-Cutting Features"
        AK[Transaction Manager] -.-> K
        AL[Query Optimizer] -.-> K
        AM[Backup Manager] -.-> K
        AN[Replication Manager] -.-> K
        AO[Encryption Manager] -.-> M
        AP[Compression Manager] -.-> M
    end

### 支持的存储后端
- **PostgreSQL**: 关系型数据存储
- **Redis**: 缓存和会话存储
- **SQLite**: 轻量级本地存储
- **MinIO**: 对象存储
- **本地文件系统**: 配置文件和日志存储

## 核心组件

### 1. 数据库连接管理 (Database Connection)
文件: `database.rs`
- **DatabaseManager**: 数据库连接池管理
- **ConnectionPool**: 连接池实现
- **DatabaseConfig**: 数据库配置

### 2. 实体仓库 (Entity Repository)
文件: `repository.rs`
- **UserRepository**: 用户数据仓库
- **ConfigRepository**: 配置数据仓库
- **AuditRepository**: 审计日志仓库
- **TaskRepository**: 任务数据仓库

### 3. 缓存管理 (Cache Management)
文件: `cache.rs`
- **CacheManager**: 缓存管理器
- **RedisCache**: Redis缓存实现
- **MemoryCache**: 内存缓存实现
- **CacheStrategy**: 缓存策略

### 4. 文件存储 (File Storage)
文件: `file_storage.rs`
- **FileStorage**: 文件存储服务
- **StorageBackend**: 存储后端接口
- **LocalStorage**: 本地文件存储
- **S3Storage**: S3兼容存储

### 5. 数据迁移 (Migration)
文件: `migration.rs`
- **MigrationManager**: 迁移管理器
- **Migration**: 迁移定义
- **Schema**: 数据库模式定义

## 数据模型

### 用户表 (users)
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'viewer',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### 配置表 (configurations)
```sql
CREATE TABLE configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(255) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    type VARCHAR(20) NOT NULL DEFAULT 'string',
    scope VARCHAR(20) NOT NULL DEFAULT 'global',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### 审计日志表 (audit_logs)
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(100) NOT NULL,
    details JSONB,
    level VARCHAR(20) NOT NULL DEFAULT 'info',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### 任务表 (scheduled_tasks)
```sql
CREATE TABLE scheduled_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    cron_expression VARCHAR(100) NOT NULL,
    command TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    last_run TIMESTAMP WITH TIME ZONE,
    next_run TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

## 存储配置

### 数据库配置
```yaml
storage:
  database:
    type: postgresql
    host: localhost
    port: 5432
    database: galaxy_ops
    username: orion
    password: secret
    pool_size: 10
    ssl_mode: prefer
```

### Redis配置
```yaml
storage:
  redis:
    host: localhost
    port: 6379
    database: 0
    password: secret
    pool_size: 10
```

### 文件存储配置
```yaml
storage:
  file:
    backend: local
    local:
      path: /var/lib/galaxy-ops/data
    s3:
      endpoint: https://s3.amazonaws.com
      bucket: galaxy-ops
      region: us-east-1
      access_key: AKIA...
      secret_key: secret
```

## 使用示例

### 数据库操作
```rust
use galaxy_ops::storage::{DatabaseManager, UserRepository};

let db = DatabaseManager::new(config).await?;
let user_repo = UserRepository::new(db.clone());

// 创建用户
let user = user_repo.create_user("admin", "admin@example.com", "hashed_password").await?;

// 查询用户
let user = user_repo.find_by_username("admin").await?;
let users = user_repo.list_users(0, 10).await?;
```

### 缓存操作
```rust
use galaxy_ops::storage::{CacheManager, CacheStrategy};

let cache = CacheManager::new(redis_config).await?;

// 设置缓存
cache.set("user:admin", &user, Duration::from_secs(3600)).await?;

// 获取缓存
let user: Option<User> = cache.get("user:admin").await?;

// 删除缓存
cache.delete("user:admin").await?;
```

### 文件存储操作
```rust
use galaxy_ops::storage::{FileStorage, StorageBackend};

let storage = FileStorage::new(config).await?;

// 上传文件
storage.upload("config/app.yaml", config_content.as_bytes()).await?;

// 下载文件
let content = storage.download("config/app.yaml").await?;

// 删除文件
storage.delete("config/app.yaml").await?;
```

## 事务管理

### 事务支持
```rust
use galaxy_ops::storage::DatabaseManager;

let db = DatabaseManager::new(config).await?;
let tx = db.begin_transaction().await?;

// 在事务中执行操作
user_repo.create_user_in_tx(&tx, "user1", "email1", "pass1").await?;
config_repo.set_config_in_tx(&tx, "key1", "value1").await?;

// 提交事务
tx.commit().await?;
```

## 数据迁移

### 迁移脚本
```rust
use galaxy_ops::storage::MigrationManager;

let migrator = MigrationManager::new(db).await?;
migrator.run_migrations().await?;
```

### 创建新迁移
```sql
-- migrations/001_create_users_table.sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'viewer',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
```

## 性能优化

### 索引策略
- 用户表：username, email索引
- 审计日志：user_id, created_at索引
- 配置表：key索引

### 查询优化
- 使用预编译语句
- 批量操作
- 分页查询
- 延迟加载

### 缓存策略
- 读多写少的数据使用缓存
- 设置合理的TTL
- 使用缓存预热
- 实现缓存失效机制

## 备份与恢复

### 数据库备份
```bash
# PostgreSQL备份
pg_dump -h localhost -U orion galaxy_ops > backup.sql

# Redis备份
redis-cli BGSAVE
```

### 数据恢复
```bash
# PostgreSQL恢复
psql -h localhost -U orion galaxy_ops < backup.sql

# Redis恢复
redis-cli --rdb dump.rdb
```

## 监控与告警

### 数据库监控
- 连接池状态
- 查询性能
- 慢查询日志
- 死锁检测

### 缓存监控
- 命中率
- 内存使用
- 键过期统计
- 连接状态

### 存储监控
- 磁盘空间使用
- 文件操作统计
- 上传下载速率
- 错误率

## 安全配置

### 数据库安全
- 使用SSL连接
- 强密码策略
- 最小权限原则
- 定期更新密码

### 数据加密
- 敏感数据加密存储
- 传输加密
- 备份加密
- 密钥管理

## 扩展性设计

### 分库分表
- 按用户ID分片
- 按时间分片
- 读写分离
- 主从复制

### 缓存集群
- Redis集群
- 一致性哈希
- 故障转移
- 数据同步

## 测试策略

### 单元测试
- Repository层测试
- 数据库操作测试
- 缓存操作测试
- 文件存储测试

### 集成测试
- 端到端测试
- 性能测试
- 并发测试
- 故障恢复测试

## 最佳实践

1. **连接管理**: 使用连接池，避免频繁创建连接
2. **错误处理**: 提供详细的错误信息，便于调试
3. **日志记录**: 记录关键操作和错误，便于审计
4. **性能监控**: 监控查询性能和资源使用
5. **数据一致性**: 使用事务保证数据一致性
6. **备份策略**: 定期备份，测试恢复流程
7. **安全考虑**: 输入验证、权限检查、敏感信息脱敏