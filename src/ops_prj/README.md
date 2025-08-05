# Ops Project 模块

## 概述

Ops Project 模块是 galaxy-ops 项目的核心组件，负责项目生命周期管理、配置管理和系统初始化。该模块提供了完整的项目操作框架，支持从项目创建到部署的全流程管理。

## 系统架构

```
ops_prj/
├── conf.rs          # 项目配置管理
├── import.rs        # 项目导入功能
├── init.rs          # 项目初始化
├── mod.rs           # 模块入口
├── proj.rs          # 项目核心逻辑
└── system.rs        # 系统级操作
```

## 核心组件

### 1. 项目配置管理 (conf.rs)

负责项目配置的加载、验证和管理，支持多种配置格式（YAML、JSON、TOML）。

**主要功能：**
- 配置模板管理
- 环境变量解析
- 配置验证和校验
- 配置版本控制

**数据结构：**
```rust
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub environments: HashMap<String, EnvironmentConfig>,
    pub dependencies: Vec<Dependency>,
    pub build: BuildConfig,
    pub deploy: DeployConfig,
}

pub struct EnvironmentConfig {
    pub name: String,
    pub variables: HashMap<String, String>,
    pub resources: ResourceConfig,
    pub secrets: HashMap<String, SecretConfig>,
}
```

### 2. 项目导入 (import.rs)

支持从多种源导入现有项目，包括Git仓库、本地目录、压缩包等。

**支持的导入源：**
- Git仓库（GitHub、GitLab、Bitbucket）
- 本地文件系统
- 压缩包（ZIP、TAR.GZ）
- Docker镜像
- Helm Charts

**导入流程：**
```rust
pub struct ImportManager {
    pub source: ImportSource,
    pub destination: PathBuf,
    pub options: ImportOptions,
}

pub enum ImportSource {
    Git { url: String, branch: Option<String> },
    Local { path: PathBuf },
    Archive { path: PathBuf, format: ArchiveFormat },
    Docker { image: String, tag: String },
    Helm { chart: String, version: String },
}
```

### 3. 项目初始化 (init.rs)

提供项目初始化功能，支持多种项目模板和脚手架。

**初始化模板：**
- Rust项目模板
- Node.js项目模板
- Python项目模板
- Go项目模板
- Java项目模板
- 微服务模板
- Web应用模板

**初始化配置：**
```rust
pub struct InitConfig {
    pub template: ProjectTemplate,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}

pub enum ProjectTemplate {
    RustLibrary,
    RustBinary,
    NodeJsApp,
    PythonPackage,
    GoModule,
    JavaMaven,
    JavaGradle,
    Microservice,
    WebApp,
    Custom(String),
}
```

### 4. 项目核心逻辑 (proj.rs)

实现项目的核心管理功能，包括项目创建、更新、删除、查询等操作。

**核心功能：**
- 项目生命周期管理
- 项目状态跟踪
- 项目依赖管理
- 项目版本控制
- 项目权限管理

**项目状态：**
```rust
pub enum ProjectStatus {
    Initialized,
    Configured,
    Building,
    Testing,
    Deploying,
    Running,
    Stopped,
    Failed,
    Archived,
}

pub struct Project {
    pub id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub config: ProjectConfig,
    pub metadata: ProjectMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 5. 系统级操作 (system.rs)

提供系统级的项目操作功能，包括环境管理、资源分配、监控等。

**系统功能：**
- 环境生命周期管理
- 资源分配和回收
- 系统监控和告警
- 性能优化
- 故障恢复

**系统配置：**
```rust
pub struct SystemConfig {
    pub environments: Vec<Environment>,
    pub resource_limits: ResourceLimits,
    pub monitoring: MonitoringConfig,
    pub backup: BackupConfig,
    pub security: SecurityConfig,
}

pub struct Environment {
    pub name: String,
    pub type_: EnvironmentType,
    pub resources: ResourceAllocation,
    pub variables: HashMap<String, String>,
    pub health_checks: Vec<HealthCheck>,
}
```

## 使用示例

### 1. 创建新项目

```bash
# 使用CLI创建项目
gops project create --name my-app --template rust-binary

# 指定详细配置
gops project create \
    --name my-app \
    --template rust-binary \
    --description "A sample Rust application" \
    --author "John Doe" \
    --license MIT \
    --dependency serde \
    --dependency tokio
```

### 2. 导入现有项目

```bash
# 从Git仓库导入
gops project import --source git --url https://github.com/user/repo.git

# 从本地目录导入
gops project import --source local --path /path/to/project

# 从Docker镜像导入
gops project import --source docker --image my-app:latest
```

### 3. 程序化使用

```rust
use galaxy_ops::ops_prj::{ProjectManager, InitConfig, ProjectTemplate};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ProjectManager::new();
    
    // 创建新项目
    let config = InitConfig {
        template: ProjectTemplate::RustBinary,
        name: "my-app".to_string(),
        description: Some("A sample Rust application".to_string()),
        author: Some("John Doe".to_string()),
        license: Some("MIT".to_string()),
        dependencies: vec!["serde".to_string(), "tokio".to_string()],
        features: vec!["full".to_string()],
    };
    
    let project = manager.create_project(config).await?;
    println!("Created project: {}", project.name);
    
    // 导入现有项目
    let imported = manager
        .import_project()
        .from_git("https://github.com/user/repo.git")
        .branch("main")
        .destination("/projects/imported")
        .execute()
        .await?;
    
    println!("Imported project: {}", imported.name);
    
    // 获取项目列表
    let projects = manager.list_projects().await?;
    for project in projects {
        println!("Project: {} - {:?}", project.name, project.status);
    }
    
    Ok(())
}
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let config = InitConfig {
            template: ProjectTemplate::RustBinary,
            name: "test-app".to_string(),
            description: None,
            author: None,
            license: None,
            dependencies: vec![],
            features: vec![],
        };
        
        let project = Project::create(config).unwrap();
        assert_eq!(project.name, "test-app");
        assert_eq!(project.status, ProjectStatus::Initialized);
    }

    #[tokio::test]
    async fn test_project_import() {
        let manager = ProjectManager::new();
        let result = manager
            .import_project()
            .from_local("/tmp/test-project")
            .execute()
            .await;
        
        assert!(result.is_ok());
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_project_lifecycle() {
        let manager = ProjectManager::new();
        
        // 创建项目
        let config = InitConfig {
            template: ProjectTemplate::RustBinary,
            name: "integration-test".to_string(),
            description: Some("Integration test project".to_string()),
            author: Some("Test".to_string()),
            license: Some("MIT".to_string()),
            dependencies: vec!["serde".to_string()],
            features: vec!["default".to_string()],
        };
        
        let project = manager.create_project(config).await.unwrap();
        assert_eq!(project.name, "integration-test");
        
        // 更新项目
        let updated = manager
            .update_project(&project.id)
            .description("Updated description")
            .execute()
            .await
            .unwrap();
        
        assert_eq!(updated.description.unwrap(), "Updated description");
        
        // 删除项目
        manager.delete_project(&project.id).await.unwrap();
        
        let deleted = manager.get_project(&project.id).await;
        assert!(deleted.is_err());
    }
}
```

## 最佳实践

### 1. 项目结构设计
- **模块化**: 将项目分解为独立的模块
- **可配置**: 使用配置文件管理项目设置
- **可测试**: 设计易于测试的项目结构
- **可扩展**: 预留扩展点和接口

### 2. 依赖管理
- **版本锁定**: 使用锁文件确保依赖版本一致性
- **安全扫描**: 定期扫描依赖安全漏洞
- **最小依赖**: 只引入必要的依赖
- **更新策略**: 制定依赖更新策略

### 3. 环境管理
- **环境隔离**: 使用容器或虚拟环境隔离
- **配置管理**: 使用环境变量管理配置
- **密钥管理**: 使用安全的密钥存储
- **监控告警**: 设置环境健康监控

### 4. 持续集成
- **自动化测试**: 设置自动化测试流水线
- **代码质量**: 集成代码质量检查工具
- **安全扫描**: 集成安全扫描工具
- **部署自动化**: 实现自动化部署

## 扩展性

### 1. 自定义模板
```rust
impl ProjectTemplateProvider for CustomTemplateProvider {
    fn get_template(&self, name: &str) -> Option<ProjectTemplate> {
        match name {
            "custom-rust" => Some(ProjectTemplate::Custom("rust-custom".to_string())),
            "custom-node" => Some(ProjectTemplate::Custom("node-custom".to_string())),
            _ => None,
        }
    }
}
```

### 2. 自定义导入源
```rust
impl ImportSourceProvider for CustomImportSourceProvider {
    fn create_source(&self, config: &ImportConfig) -> Result<ImportSource, ImportError> {
        match config.source_type.as_str() {
            "custom-git" => Ok(ImportSource::CustomGit {
                url: config.url.clone(),
                branch: config.branch.clone(),
                token: config.token.clone(),
            }),
            _ => Err(ImportError::UnsupportedSource),
        }
    }
}
```

### 3. 自定义构建系统
```rust
impl BuildSystem for CustomBuildSystem {
    async fn build(&self, project: &Project) -> Result<BuildResult, BuildError> {
        // 实现自定义构建逻辑
        match project.config.build.system.as_str() {
            "custom" => self.execute_custom_build(project).await,
            _ => Err(BuildError::UnsupportedSystem),
        }
    }
}
```

## 相关模块

- **module**: 模块管理和依赖关系
- **package**: 包管理和版本控制
- **system**: 系统级配置和管理
- **workflow**: 工作流管理和自动化
- **storage**: 数据持久化和存储管理

## 故障排除

### 常见问题

1. **项目创建失败**
   - 检查模板是否存在
   - 验证磁盘空间
   - 检查权限设置

2. **导入失败**
   - 验证网络连接
   - 检查认证信息
   - 确认源路径有效

3. **构建失败**
   - 检查依赖安装
   - 验证构建工具
   - 查看构建日志

### 调试工具

```bash
# 查看项目状态
gops project status --name my-app

# 查看项目日志
gops project logs --name my-app

# 调试项目配置
gops project config --name my-app --validate

# 清理项目缓存
gops project clean --name my-app
```

## 性能优化

### 1. 缓存策略
- **模板缓存**: 缓存常用模板
- **依赖缓存**: 缓存依赖包
- **构建缓存**: 缓存构建结果
- **配置缓存**: 缓存配置解析结果

### 2. 并发处理
- **并行导入**: 支持多个项目并行导入
- **异步构建**: 使用异步构建提高性能
- **资源池化**: 使用连接池优化资源使用

### 3. 监控指标
- **项目数量**: 监控项目总数
- **构建时间**: 监控平均构建时间
- **成功率**: 监控项目操作成功率
- **资源使用**: 监控系统资源使用情况