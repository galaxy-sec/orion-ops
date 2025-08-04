# Package 模块文档

## 概述
Package模块是Orion Ops的包管理系统，负责软件包的创建、解析、验证和管理。它提供了统一的包格式定义、版本控制、依赖解析和包仓库管理功能。

## 包架构

### 包管理设计
```
┌─────────────────┐
│   Package       │  ← 包定义
├─────────────────┤
│   Registry      │  ← 包仓库
├─────────────────┤
│   Resolver      │  ← 依赖解析
└─────────────────┘
```

### 包管理系统架构图

```mermaid
graph TB
    subgraph "Package Management Core"
        A[PackageManager] --> B[PackageDefinition]
        A --> C[RegistryManager]
        A --> D[DependencyResolver]
        A --> E[PackageValidator]
        A --> F[PackageInstaller]
        
        B --> G[Package]
        B --> H[PackageMetadata]
        B --> I[PackageVersion]
        B --> J[PackageDependency]
        B --> K[PackageManifest]
        
        C --> L[LocalRegistry]
        C --> M[RemoteRegistry]
        C --> N[RegistryIndex]
        
        D --> O[DependencyGraph]
        D --> P[VersionSolver]
        D --> Q[ConflictResolver]
        
        E --> R[ChecksumValidator]
        E --> S[SignatureValidator]
        E --> T[SecurityScanner]
        
        F --> U[PackageExtractor]
        F --> V[FileInstaller]
        F --> W[PostInstallHook]
    end
    
    subgraph "Package Types"
        G --> G1[Application Package]
        G --> G2[Library Package]
        G --> G3[Module Package]
        G --> G4[Template Package]
    end
    
    subgraph "Registry Types"
        L --> L1[Local Cache]
        L --> L2[Local Index]
        M --> M1[Git Registry]
        M --> M2[HTTP Registry]
        M --> M3[Private Registry]
    end
    
    subgraph "Dependency Types"
        J --> J1[Runtime Dependency]
        J --> J2[Development Dependency]
        J --> J3[Optional Dependency]
        J --> J4[Peer Dependency]
    end
    
    subgraph "Validation Steps"
        R --> R1[SHA256 Check]
        R --> R2[MD5 Check]
        S --> S1[GPG Signature]
        S --> S2[Certificate Validation]
        T --> T1[Vulnerability Scan]
        T --> T2[License Check]
    end
    
    subgraph "Package Lifecycle"
        A --> X[Package Creation]
        A --> Y[Package Publishing]
        A --> Z[Package Installation]
        A --> AA[Package Update]
        A --> AB[Package Removal]
    end
    
    subgraph "Storage Backends"
        L --> AC[File System]
        M --> AD[HTTP Server]
        M --> AE[Git Repository]
        M --> AF[Cloud Storage]
    end

### 核心职责
- **包定义**: 定义统一的包格式和元数据
- **版本管理**: 语义化版本控制和兼容性检查
- **依赖解析**: 自动解析和安装依赖包
- **包验证**: 包完整性和安全性验证
- **仓库管理**: 本地和远程包仓库管理

## 核心组件

### 1. 包类型定义 (Package Types)
文件: `types.rs`
- **Package**: 包结构定义
- **PackageMetadata**: 包元数据
- **PackageVersion**: 包版本信息
- **PackageDependency**: 包依赖定义
- **PackageManifest**: 包清单文件

#### 包结构定义
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub metadata: PackageMetadata,
    pub dependencies: Vec<PackageDependency>,
    pub files: Vec<PackageFile>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version_req: VersionReq,
    pub optional: bool,
    pub features: Vec<String>,
    pub registry: Option<String>,
}
```

### 2. 包管理器 (Package Manager)
文件: `mod.rs`
- **PackageManager**: 包管理器主类
- **PackageRegistry**: 包仓库接口
- **PackageResolver**: 依赖解析器
- **PackageInstaller**: 包安装器
- **PackageValidator**: 包验证器

#### 包管理器功能
```rust
use galaxy_ops::package::{PackageManager, PackageRegistry};

let manager = PackageManager::new();

// 安装包
manager.install("example-package", "^1.0.0").await?;

// 卸载包
manager.uninstall("example-package").await?;

// 更新包
manager.update("example-package").await?;

// 列出已安装包
let installed = manager.list_installed().await?;

// 搜索包
let results = manager.search("web framework").await?;
```

## 包格式规范

### 包清单文件 (package.yml)
```yaml
# package.yml
name: "example-package"
version: "1.2.3"
description: "An example package for demonstration"
author: "John Doe <john@example.com>"
license: "MIT"
homepage: "https://example.com"
repository: "https://github.com/example/example-package"

keywords:
  - example
  - demo
  - package

categories:
  - development
  - utilities

dependencies:
  serde:
    version: "^1.0"
    features: ["derive"]
  tokio:
    version: "^1.0"
    features: ["full"]
  
optional_dependencies:
  tracing:
    version: "^0.1"
    optional: true

files:
  - src/
  - Cargo.toml
  - README.md
  - LICENSE

checksum:
  sha256: "a1b2c3d4e5f6..."

build:
  script: "cargo build --release"
  target: "target/release/example-package"

scripts:
  test: "cargo test"
  build: "cargo build --release"
  publish: "cargo publish"
```

### 版本规范
- **语义化版本**: 遵循 SemVer 规范 (MAJOR.MINOR.PATCH)
- **版本范围**: 支持 ^、~、>=、<= 等版本约束
- **预发布版本**: 支持 alpha、beta、rc 等预发布标识
- **构建元数据**: 支持构建号和提交哈希

### 依赖解析规则
```rust
// 版本约束示例
"^1.2.3"  // 兼容 1.2.3 及以上，但小于 2.0.0
"~1.2.3"  // 兼容 1.2.3 及以上，但小于 1.3.0
">=1.2.3" // 大于等于 1.2.3
"<2.0.0"  // 小于 2.0.0
"1.2.*"   // 1.2.x 的任意版本
```

## 包仓库管理

### 本地仓库
```rust
use galaxy_ops::package::{LocalRegistry, PackageRegistry};

let registry = LocalRegistry::new("/var/lib/galaxy-ops/packages");

// 添加包到本地仓库
registry.publish(package).await?;

// 从本地仓库获取包
let package = registry.get("example-package", "1.2.3").await?;

// 列出本地包
let packages = registry.list().await?;

// 删除本地包
registry.yank("example-package", "1.2.3").await?;
```

### 远程仓库
```rust
use galaxy_ops::package::{RemoteRegistry, PackageRegistry};

let registry = RemoteRegistry::new("https://registry.example.com");

// 搜索远程包
let results = registry.search("web").await?;

// 下载包
let package = registry.download("example-package", "1.2.3").await?;

// 获取包信息
let info = registry.info("example-package").await?;
```

### 多仓库配置
```yaml
# registry.yml
registries:
  official:
    url: "https://registry.galaxy-ops.com"
    priority: 1
  
  company:
    url: "https://packages.company.com"
    priority: 2
    auth:
      token: "${COMPANY_TOKEN}"
  
  local:
    path: "/var/lib/galaxy-ops/packages"
    priority: 3
```

## 包验证机制

### 完整性验证
```rust
impl PackageValidator {
    pub fn validate_integrity(&self, package: &Package) -> Result<(), ValidationError> {
        // 验证包结构
        self.validate_structure(package)?;
        
        // 验证文件完整性
        self.validate_files(package)?;
        
        // 验证校验和
        self.validate_checksum(package)?;
        
        // 验证签名
        self.validate_signature(package)?;
        
        Ok(())
    }
}
```

### 安全性验证
```rust
impl PackageValidator {
    pub fn validate_security(&self, package: &Package) -> Result<(), ValidationError> {
        // 验证依赖安全性
        self.validate_dependencies(package)?;
        
        // 验证文件权限
        self.validate_permissions(package)?;
        
        // 验证脚本安全性
        self.validate_scripts(package)?;
        
        Ok(())
    }
}
```

## 包生命周期

### 生命周期状态
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageState {
    Published,      // 已发布
    Installed,      // 已安装
    Updated,        // 已更新
    Deprecated,     // 已弃用
    Yanked,         // 已撤回
    Uninstalled,    // 已卸载
}
```

### 生命周期管理
```rust
impl PackageManager {
    pub async fn lifecycle(&self, package: &str) -> Result<PackageLifecycle, Error> {
        let lifecycle = PackageLifecycle::new(package);
        
        // 检查包状态
        let state = lifecycle.current_state().await?;
        
        // 获取版本历史
        let history = lifecycle.version_history().await?;
        
        // 获取依赖关系
        let dependencies = lifecycle.dependencies().await?;
        
        Ok(PackageLifecycle {
            state,
            history,
            dependencies,
        })
    }
}
```

## 包缓存机制

### 缓存策略
```rust
use galaxy_ops::package::{PackageCache, CacheStrategy};

let cache = PackageCache::new("/var/cache/galaxy-ops/packages");

// 设置缓存策略
cache.set_strategy(CacheStrategy::LRU {
    max_size: 1024 * 1024 * 1024, // 1GB
    max_age: Duration::from_secs(3600), // 1小时
});

// 缓存包
cache.put(package).await?;

// 从缓存获取
let package = cache.get("example-package", "1.2.3").await?;

// 清理过期缓存
cache.cleanup().await?;
```

### 缓存层次
- **内存缓存**: 热数据快速访问
- **磁盘缓存**: 持久化缓存存储
- **网络缓存**: CDN加速下载
- **分布式缓存**: 多节点共享缓存

## 包构建系统

### 构建配置
```yaml
# build.yml
build:
  target: "x86_64-unknown-linux-gnu"
  profile: "release"
  features: ["default"]
  
  steps:
    - name: "compile"
      command: "cargo build --release"
    
    - name: "test"
      command: "cargo test"
    
    - name: "package"
      command: "tar -czf package.tar.gz target/release/"
  
  artifacts:
    - "target/release/example-package"
    - "README.md"
    - "LICENSE"
```

### 构建脚本
```rust
use galaxy_ops::package::{PackageBuilder, BuildConfig};

let builder = PackageBuilder::new();
let config = BuildConfig::from_file("build.yml")?;

// 执行构建
let result = builder.build(&config).await?;

// 验证构建结果
let package = builder.validate(result).await?;

// 生成包清单
let manifest = builder.generate_manifest(&package)?;
```

## 包发布流程

### 发布步骤
1. **验证包**: 检查包完整性和安全性
2. **版本检查**: 验证版本号是否符合规范
3. **依赖检查**: 确保所有依赖可用
4. **构建包**: 生成最终的包文件
5. **签名包**: 使用私钥签名包
6. **上传包**: 上传到包仓库
7. **发布通知**: 通知订阅者包已发布

### 发布配置
```yaml
# publish.yml
publish:
  registry: "https://registry.galaxy-ops.com"
  auth:
    token: "${REGISTRY_TOKEN}"
  
  validation:
    strict: true
    security_scan: true
    dependency_check: true
  
  notification:
    email: ["maintainers@example.com"]
    webhook: "https://hooks.example.com/package-published"
```

## 包搜索和发现

### 搜索功能
```rust
use galaxy_ops::package::{PackageSearch, SearchQuery};

let search = PackageSearch::new();

// 基本搜索
let results = search
    .query("web framework")
    .limit(10)
    .execute()
    .await?;

// 高级搜索
let results = search
    .query("database")
    .category("development")
    .author("trusted")
    .min_downloads(1000)
    .sort_by("downloads")
    .execute()
    .await?;
```

### 搜索索引
```rust
#[derive(Debug, Clone)]
pub struct SearchIndex {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub author: String,
    pub downloads: u64,
    pub rating: f64,
    pub updated: DateTime<Utc>,
}
```

## 包依赖图

### 依赖关系可视化
```rust
use galaxy_ops::package::{DependencyGraph, GraphVisualizer};

let graph = DependencyGraph::build("example-package").await?;

// 获取依赖树
let tree = graph.get_dependency_tree();

// 检测循环依赖
let cycles = graph.detect_cycles();
if !cycles.is_empty() {
    println!("发现循环依赖: {:?}", cycles);
}

// 可视化依赖图
let dot = GraphVisualizer::to_dot(&graph);
println!("依赖图: {}", dot);
```

### 依赖冲突解决
```rust
impl DependencyResolver {
    pub fn resolve_conflicts(&self, conflicts: &[DependencyConflict]) -> Resolution {
        // 版本冲突解决策略
        let strategy = ResolutionStrategy::new()
            .prefer_latest()
            .allow_pre_release(false)
            .max_depth(10);
        
        strategy.resolve(conflicts)
    }
}
```

## 包测试框架

### 测试类型
- **单元测试**: 包内函数的测试
- **集成测试**: 包间交互的测试
- **兼容性测试**: 不同环境下的兼容性
- **性能测试**: 包性能基准测试
- **安全测试**: 安全漏洞扫描

### 测试配置
```yaml
# test.yml
test:
  unit:
    enabled: true
    coverage: 80
  
  integration:
    enabled: true
    environments: ["linux", "macos", "windows"]
  
  compatibility:
    enabled: true
    versions: ["1.0", "1.1", "1.2"]
  
  performance:
    enabled: true
    benchmarks: ["startup", "memory", "throughput"]
  
  security:
    enabled: true
    scanners: ["safety", "bandit", "gosec"]
```

## 包安全性

### 安全扫描
```rust
use galaxy_ops::package::{SecurityScanner, ScanResult};

let scanner = SecurityScanner::new();

// 扫描包
let result = scanner.scan_package("example-package").await?;

// 检查漏洞
if result.has_vulnerabilities() {
    for vuln in result.vulnerabilities {
        println!("发现漏洞: {}", vuln.description);
    }
}

// 生成安全报告
let report = scanner.generate_report(&result)?;
```

### 签名验证
```rust
impl PackageSigner {
    pub fn sign_package(&self, package: &Package, key: &PrivateKey) -> Result<Signature, Error> {
        // 使用私钥签名包
        let signature = key.sign(&package.checksum)?;
        Ok(signature)
    }
    
    pub fn verify_signature(&self, package: &Package, signature: &Signature, key: &PublicKey) -> Result<bool, Error> {
        // 使用公钥验证签名
        key.verify(&package.checksum, signature)
    }
}
```

## 包统计和分析

### 使用统计
```rust
#[derive(Debug, Clone)]
pub struct PackageStats {
    pub downloads: u64,
    pub installs: u64,
    pub updates: u64,
    pub uninstalls: u64,
    pub rating: f64,
    pub reviews: u64,
    pub issues: u64,
}

impl PackageAnalytics {
    pub async fn get_stats(&self, package: &str) -> Result<PackageStats, Error> {
        let stats = self.database.get_package_stats(package).await?;
        Ok(stats)
    }
    
    pub async fn get_trending(&self, period: Duration) -> Result<Vec<Package>, Error> {
        let trending = self.database.get_trending_packages(period).await?;
        Ok(trending)
    }
}
```

## 包迁移和升级

### 版本迁移
```rust
impl PackageMigrator {
    pub async fn migrate(&self, from: &str, to: &str) -> Result<MigrationPlan, Error> {
        let plan = MigrationPlan::new(from, to);
        
        // 检查兼容性
        plan.check_compatibility().await?;
        
        // 生成迁移步骤
        plan.generate_steps().await?;
        
        // 执行迁移
        plan.execute