# 核心文件文档

## 概述

本目录包含了 galaxy-ops 项目的核心顶层文件，这些文件定义了系统的基本类型、错误处理、配置管理和工具函数。这些文件构成了整个系统的基础架构。

## 文件结构

### 1. types.rs - 核心类型定义

**文件路径**: `src/types.rs`
**作用**: 定义系统使用的核心类型和trait

**核心类型：**
- `SysUpdateValue`: 系统更新值类型，包含变量集合
- `LocalizeOptions`: 本地化配置选项
- `ValuePath`: 路径处理相关类型

**核心trait：**
- `SysUpdateable`: 系统更新trait，定义了本地更新接口
- `Localizable`: 本地化trait（通过代码推断）

**使用示例：**
```rust
use galaxy_ops::types::{SysUpdateValue, LocalizeOptions, SysUpdateable};

// 创建系统更新值
let vars = VarCollection::new();
let update_value = SysUpdateValue::new(vars);

// 创建本地化选项
let options = LocalizeOptions::new(global_dict, true);
```

### 2. error.rs - 错误处理

**文件路径**: `src/error.rs`
**作用**: 定义系统统一的错误类型和处理机制

**错误类型层次：**
- `MainReason`: 主错误类型枚举
  - `Localize`: 本地化相关错误
  - `Element`: 元素相关错误
  - `Mod`: 模块相关错误
  - `Sys`: 系统相关错误
  - `Ops`: 操作相关错误
  - `Uvs`: 通用错误

**具体错误枚举：**
- `ElementReason`: 元素错误（如缺失）
- `ModReason`: 模块错误（加载、保存、更新失败）
- `SysReason`: 系统错误（加载、保存、更新失败）
- `OpsReason`: 操作错误
- `LocalizeReason`: 本地化错误

**使用示例：**
```rust
use galaxy_ops::error::{MainReason, MainResult};

fn example_function() -> MainResult<String> {
    // 返回成功结果
    Ok("success".to_string())

    // 返回错误
    Err(MainReason::Element(ElementReason::Miss("required_field".to_string())).into())
}
```

### 3. conf.rs - 配置管理

**文件路径**: `src/conf.rs`
**作用**: 管理系统配置，包括应用配置、模块配置、环境配置等

**预期结构：**
```rust
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub modules: ModuleConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

pub struct ModuleConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    pub configurations: HashMap<String, ValueDict>,
}
```

### 4. const_vars.rs - 常量定义

**文件路径**: `src/const_vars.rs`
**作用**: 定义系统使用的全局常量和默认值

**预期内容：**
- 系统常量（版本、名称、作者）
- 文件路径常量
- 配置键常量
- 默认值定义
- 环境变量键

### 5. infra.rs - 基础设施

**文件路径**: `src/infra.rs`
**作用**: 提供基础设施相关的功能和工具

**预期功能：**
- 网络配置管理
- 存储初始化
- 服务发现
- 负载均衡配置
- 基础设施抽象

### 6. predule.rs - 预导入模块

**文件路径**: `src/predule.rs`
**作用**: 提供常用的预导入模块和类型别名

**预期内容：**
```rust
pub use crate::error::{MainReason, MainResult};
pub use crate::types::*;
pub use crate::const_vars::*;
pub use orion_variate::vars::{ValueDict, VarCollection};
```

### 7. resource.rs - 资源管理

**文件路径**: `src/resource.rs`
**作用**: 管理系统资源，包括计算资源、存储资源、网络资源等

**预期结构：**
```rust
pub struct ResourceManager {
    pub compute: ComputeResource,
    pub storage: StorageResource,
    pub network: NetworkResource,
}

pub struct ComputeResource {
    pub cpu: CpuConfig,
    pub memory: MemoryConfig,
    pub gpu: Option<GpuConfig>,
}
```

### 8. software.rs - 软件管理

**文件路径**: `src/software.rs`
**作用**: 管理软件包、依赖、版本等

**预期功能：**
- 软件包安装和卸载
- 版本管理
- 依赖解析
- 软件仓库管理

### 9. spec.rs - 规范定义

**文件路径**: `src/spec.rs`
**作用**: 定义系统规范和标准

**预期内容：**
- API规范
- 数据格式规范
- 配置规范
- 接口规范

### 10. task.rs - 任务管理

**文件路径**: `src/task.rs`
**作用**: 管理系统任务和作业

**预期结构：**
```rust
pub struct TaskManager {
    pub tasks: Vec<Task>,
    pub scheduler: TaskScheduler,
}

pub struct Task {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
}
```

### 11. tools.rs - 工具函数

**文件路径**: `src/tools.rs`
**作用**: 提供通用的工具函数和辅助方法

**预期功能：**
- 文件操作工具
- 字符串处理工具
- 时间处理工具
- 加密解密工具
- 网络工具

## 使用指南

### 1. 基础使用

```rust
use galaxy_ops::{
    types::*,
    error::*,
    conf::AppConfig,
    tools::*,
};

#[tokio::main]
async fn main() -> MainResult<()> {
    // 加载配置
    let config = AppConfig::load("config.yaml").await?;

    // 初始化系统
    let system = System::new(config);

    // 执行操作
    system.run().await?;

    Ok(())
}
```

### 2. 错误处理

```rust
use galaxy_ops::error::{MainReason, MainResult};

fn handle_error() -> MainResult<()> {
    match risky_operation() {
        Ok(result) => {
            println!("Success: {}", result);
            Ok(())
        }
        Err(e) => {
            match e.reason() {
                MainReason::Element(ElementReason::Miss(field)) => {
                    println!("Missing required field: {}", field);
                }
                MainReason::Sys(SysReason::Load) => {
                    println!("Failed to load system configuration");
                }
                _ => {
                    println!("Unknown error: {}", e);
                }
            }
            Err(e)
        }
    }
}
```

### 3. 配置管理

```rust
use galaxy_ops::conf::AppConfig;

async fn manage_config() -> MainResult<()> {
    // 创建默认配置
    let config = AppConfig::default();

    // 从文件加载配置
    let config = AppConfig::load("config.yaml").await?;

    // 验证配置
    config.validate()?;

    // 保存配置
    config.save("config.yaml").await?;

    Ok(())
}
```

## 最佳实践

### 1. 错误处理
- 使用 `MainResult` 作为统一的返回类型
- 提供详细的错误信息
- 实现错误恢复机制
- 记录错误日志

### 2. 配置管理
- 使用类型安全的配置结构
- 提供默认值和验证
- 支持环境变量覆盖
- 实现配置热重载

### 3. 资源管理
- 使用连接池管理资源
- 实现资源清理机制
- 监控资源使用情况
- 设置资源限制

### 4. 测试策略
- 为每个文件编写单元测试
- 实现集成测试
- 使用模拟对象进行测试
- 设置持续集成

## 扩展性

### 1. 自定义类型
```rust
impl crate::types::SysUpdateable<MyType> for MyCustomType {
    async fn update_local(self, path: &Path, options: &DownloadOptions) -> MainResult<MyType> {
        // 实现自定义更新逻辑
        Ok(MyType::new())
    }
}
```

### 2. 自定义错误
```rust
#[derive(Debug, Error)]
pub enum MyError {
    #[error("custom error: {0}")]
    Custom(String),
}

impl From<MyError> for MainReason {
    fn from(error: MyError) -> Self {
        MainReason::Custom(error.to_string())
    }
}
```

### 3. 自定义配置
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MyConfig {
    pub custom_field: String,
    pub custom_settings: HashMap<String, String>,
}

impl crate::conf::Configurable for MyConfig {
    fn validate(&self) -> MainResult<()> {
        // 实现自定义验证逻辑
        Ok(())
    }
}
```

## 相关模块

- **module**: 模块管理和依赖关系
- **package**: 包管理和版本控制
- **system**: 系统级配置和管理
- **service**: 服务层抽象
- **storage**: 数据持久化和存储管理
- **workflow**: 工作流管理和自动化

## 故障排除

### 常见问题

1. **类型不匹配错误**
   - 检查类型定义
   - 验证序列化/反序列化
   - 使用类型别名

2. **配置加载失败**
   - 检查文件路径
   - 验证文件格式
   - 检查文件权限

3. **错误处理不当**
   - 使用正确的错误类型
   - 提供有意义的错误信息
   - 实现错误恢复

### 调试工具

```bash
# 查看配置
cat config.yaml

# 验证配置格式
yamllint config.yaml

# 检查文件权限
ls -la config.yaml

# 调试日志
RUST_LOG=debug cargo run
```

## 性能优化

### 1. 内存优化
- 使用引用而非复制
- 实现对象池
- 避免内存泄漏

### 2. 并发优化
- 使用异步编程
- 实现连接池
- 使用缓存

### 3. I/O优化
- 批量操作
- 异步I/O
- 缓存策略
