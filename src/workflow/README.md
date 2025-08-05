# Workflow 模块文档

## 概述
Workflow模块是Orion Ops的工作流引擎核心，负责定义、执行和管理复杂的业务流程。它提供了声明式的工作流定义、分布式任务执行、状态管理、错误处理和监控功能，支持多种工作流模式（顺序、并行、条件分支、循环等）。

## 系统架构

### 工作流引擎设计
```
┌─────────────────────┐
│   Workflow Engine   │  ← 工作流引擎
├─────────────────────┤
│   Process Manager   │  ← 流程管理器
├─────────────────────┤
│   Task Executor     │  ← 任务执行器
├─────────────────────┤
│   State Manager     │  ← 状态管理器
├─────────────────────┤
│   Event Bus         │  ← 事件总线
└─────────────────────┘
```

### 工作流引擎架构图

```mermaid
graph TB
    subgraph "Workflow Engine Core"
        A[WorkflowEngine] --> B[WorkflowDefinition]
        A --> C[ProcessManager]
        A --> D[TaskExecutor]
        A --> E[StateManager]
        A --> F[EventBus]
        
        B --> G[Workflow]
        B --> H[Process]
        B --> I[Task]
        B --> J[Transition]
        B --> K[Variable]
        
        C --> L[WorkflowInstance]
        C --> M[ProcessInstance]
        C --> N[ExecutionContext]
        
        D --> O[TaskRunner]
        D --> P[TaskScheduler]
        D --> Q[TaskMonitor]
        
        E --> R[StateTracker]
        E --> S[StateMachine]
        E --> T[StateTransition]
        
        F --> U[EventPublisher]
        F --> V[EventSubscriber]
        F --> W[EventStore]
    end
    
    subgraph "Workflow Types"
        G --> G1[Sequential]
        G --> G2[Parallel]
        G --> G3[Conditional]
        G --> G4[Loop]
    end
    
    subgraph "Task Types"
        I --> I1[Script Task]
        I --> I2[API Task]
        I --> I3[Container Task]
        I --> I4[Approval Task]
    end
    
    subgraph "Execution States"
        L --> L1[Pending]
        L --> L2[Running]
        L --> L3[Completed]
        L --> L4[Failed]
        L --> L5[Cancelled]
    end
    
    subgraph "Event Types"
        U --> U1[Workflow Started]
        U --> U2[Task Completed]
        U --> U3[Error Occurred]
        U --> U4[State Changed]
    end

### 核心职责
- **工作流定义**: 声明式工作流定义和验证
- **流程执行**: 分布式任务调度和执行
- **状态管理**: 工作流实例状态跟踪和管理
- **错误处理**: 重试、补偿、回滚等错误处理机制
- **监控告警**: 工作流执行监控和告警
- **审计追踪**: 完整的执行历史和审计日志

## 核心组件

### 1. 工作流定义 (Workflow Definition)
文件: `mod.rs`
- **Workflow**: 工作流定义结构
- **Process**: 流程定义
- **Task**: 任务定义
- **Transition**: 状态转换定义
- **Variable**: 变量定义

#### 工作流定义结构
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub processes: Vec<Process>,
    pub variables: HashMap<String, Variable>,
    pub triggers: Vec<Trigger>,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: String,
    pub name: String,
    pub type_: ProcessType,
    pub tasks: Vec<Task>,
    pub transitions: Vec<Transition>,
    pub variables: HashMap<String, Variable>,
    pub error_handling: ErrorHandling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub type_: TaskType,
    pub executor: TaskExecutor,
    pub inputs: HashMap<String, Value>,
    pub outputs: HashMap<String, String>,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    SubProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Script,
    API,
    Database,
    Message,
    Approval,
    Notification,
    Container,
    Kubernetes,
}
```

### 2. 工作流实例 (Workflow Instance)
文件: `act.rs`
- **WorkflowInstance**: 工作流实例
- **ProcessInstance**: 流程实例
- **TaskInstance**: 任务实例
- **ExecutionContext**: 执行上下文
- **State**: 状态定义

#### 工作流实例管理
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub status: InstanceStatus,
    pub current_state: String,
    pub variables: HashMap<String, Value>,
    pub processes: Vec<ProcessInstance>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: InstanceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInstance {
    pub id: String,
    pub process_id: String,
    pub workflow_instance_id: String,
    pub status: ProcessStatus,
    pub current_task: Option<String>,
    pub tasks: Vec<TaskInstance>,
    pub variables: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstance {
    pub id: String,
    pub task_id: String,
    pub process_instance_id: String,
    pub status: TaskStatus,
    pub inputs: HashMap<String, Value>,
    pub outputs: HashMap<String, Value>,
    pub executor_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub logs: Vec<String>,
}
```

### 3. 项目工作流 (Project Workflow)
文件: `prj.rs`
- **ProjectWorkflow**: 项目工作流
- **ProjectTask**: 项目任务
- **ProjectTrigger**: 项目触发器
- **ProjectVariable**: 项目变量

#### 项目工作流定义
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectWorkflow {
    pub project_id: String,
    pub workflow: Workflow,
    pub triggers: Vec<ProjectTrigger>,
    pub variables: HashMap<String, ProjectVariable>,
    pub permissions: Vec<Permission>,
    pub notifications: Vec<NotificationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTrigger {
    pub id: String,
    pub name: String,
    pub type_: TriggerType,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Schedule,
    Webhook,
    GitPush,
    PullRequest,
    Issue,
    Manual,
    API,
}
```

### 4. 图形化工作流 (Graphical Workflow)
文件: `gxl.rs`
- **GraphWorkflow**: 图形化工作流
- **Node**: 节点定义
- **Edge**: 边定义
- **Graph**: 图结构

#### 图形化工作流定义
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphWorkflow {
    pub workflow: Workflow,
    pub graph: Graph,
    pub layout: Layout,
    pub styling: Styling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub type_: NodeType,
    pub position: Position,
    pub size: Size,
    pub label: String,
    pub style: NodeStyle,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub type_: EdgeType,
    pub label: Option<String>,
    pub style: EdgeStyle,
    pub conditions: Vec<Condition>,
}
```

## 工作流定义示例

### 1. 简单顺序工作流
```yaml
# simple-sequence.yml
workflow:
  id: "simple-sequence"
  name: "Simple Sequence Workflow"
  version: "1.0.0"
  
  processes:
    - id: "main"
      name: "Main Process"
      type: "sequential"
      
      tasks:
        - id: "task1"
          name: "Task 1"
          type: "script"
          executor: "bash"
          inputs:
            script: "echo 'Starting workflow'"
          
        - id: "task2"
          name: "Task 2"
          type: "api"
          executor: "http"
          inputs:
            url: "https://api.example.com/data"
            method: "GET"
          
        - id: "task3"
          name: "Task 3"
          type: "notification"
          executor: "email"
          inputs:
            to: "admin@example.com"
            subject: "Workflow completed"
            body: "The workflow has been completed successfully"
      
      transitions:
        - from: "task1"
          to: "task2"
          condition: "success"
        
        - from: "task2"
          to: "task3"
          condition: "success"
```

### 2. 并行工作流
```yaml
# parallel-workflow.yml
workflow:
  id: "parallel-workflow"
  name: "Parallel Workflow"
  version: "1.0.0"
  
  processes:
    - id: "main"
      name: "Main Process"
      type: "parallel"
      
      tasks:
        - id: "task1"
          name: "Task 1"
          type: "script"
          executor: "bash"
          inputs:
            script: "echo 'Task 1 executing'"
          
        - id: "task2"
          name: "Task 2"
          type: "script"
          executor: "bash"
          inputs:
            script: "echo 'Task 2 executing'"
          
        - id: "task3"
          name: "Task 3"
          type: "script"
          executor: "bash"
          inputs:
            script: "echo 'Task 3 executing'"
          
        - id: "join"
          name: "Join Task"
          type: "script"
          executor: "bash"
          inputs:
            script: "echo 'All tasks completed'"
      
      transitions:
        - from: "start"
          to: "task1,task2,task3"
          condition: "parallel"
        
        - from: "task1,task2,task3"
          to: "join"
          condition: "all_completed"
```

### 3. 条件分支工作流
```yaml
# conditional-workflow.yml
workflow:
  id: "conditional-workflow"
  name: "Conditional Workflow"
  version: "1.0.0"
  
  variables:
    environment:
      type: "string"
      default: "development"
    
    should_deploy:
      type: "boolean"
      default: false
  
  processes:
    - id: "main"
      name: "Main Process"
      type: "conditional"
      
      tasks:
        - id: "check_env"
          name: "Check Environment"
          type: "script"
          executor: "bash"
          inputs:
            script: |
              if [[ "${environment}" == "production" ]]; then
                echo "production=true" >> outputs
              else
                echo "production=false" >> outputs
              fi
          
        - id: "deploy_prod"
          name: "Deploy Production"
          type: "kubernetes"
          executor: "kubectl"
          inputs:
            action: "apply"
            manifest: "deployment-prod.yml"
          
        - id: "deploy_dev"
          name: "Deploy Development"
          type: "kubernetes"
          executor: "kubectl"
          inputs:
            action: "apply"
            manifest: "deployment-dev.yml"
          
        - id: "notify"
          name: "Notify Team"
          type: "notification"
          executor: "slack"
          inputs:
            channel: "#deployments"
            message: "Deployment completed for ${environment}"
      
      transitions:
        - from: "check_env"
          to: "deploy_prod"
          condition: "outputs.production == 'true'"
        
        - from: "check_env"
          to: "deploy_dev"
          condition: "outputs.production == 'false'"
        
        - from: "deploy_prod"
          to: "notify"
          condition: "success"
        
        - from: "deploy_dev"
          to: "notify"
          condition: "success"
```

## 执行引擎

### 工作流执行器
```rust
use galaxy_ops::workflow::{WorkflowEngine, WorkflowInstance};

let engine = WorkflowEngine::new();

// 创建工作流实例
let instance = engine
    .create_instance("simple-sequence")
    .variables(vec![
        ("environment", "production"),
        ("version", "1.2.3"),
    ])
    .build();

// 启动工作流
let execution_id = engine.start(instance).await?;

// 监控执行状态
let status = engine.get_status(execution_id).await?;

// 暂停工作流
engine.pause(execution_id).await?;

// 恢复工作流
engine.resume(execution_id).await?;

// 取消工作流
engine.cancel(execution_id).await?;
```

### 任务执行器
```rust
use galaxy_ops::workflow::{TaskExecutor, TaskContext};

#[async_trait]
impl TaskExecutor for MyTaskExecutor {
    async fn execute(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        match context.task_type {
            TaskType::Script => {
                let script = context.get_input("script")?;
                let output = execute_script(script).await?;
                Ok(TaskResult::new().output("result", output))
            }
            TaskType::API => {
                let url = context.get_input("url")?;
                let method = context.get_input("method")?;
                let response = make_api_call(url, method).await?;
                Ok(TaskResult::new().output("response", response))
            }
            _ => Err(TaskError::UnsupportedTaskType),
        }
    }
}
```

## 状态管理

### 状态持久化
```rust
use galaxy_ops::workflow::{StateManager, StateStore};

let state_manager = StateManager::new();

// 保存状态
state_manager.save_state(instance).await?;

// 恢复状态
let instance = state_manager.restore_state(execution_id).await?;

// 获取历史状态
let history = state_manager.get_history(execution_id).await?;

// 清理过期状态
state_manager.cleanup(Duration::from_days(7)).await?;
```

### 状态查询
```rust
// 查询运行中的实例
let running_instances = state_manager
    .query()
    .status(InstanceStatus::Running)
    .limit(100)
    .execute()
    .await?;

// 查询失败的实例
let failed_instances = state_manager
    .query()
    .status(InstanceStatus::Failed)
    .since(Duration::from_hours(24))
    .execute()
    .await?;
```

## 错误处理

### 重试策略
```yaml
# retry-policy.yml
retry_policy:
  max_attempts: 3
  backoff_strategy: "exponential"
  initial_delay: "1s"
  max_delay: "30s"
  multiplier: 2
  jitter: true
```

### 补偿机制
```rust
#[derive(Debug, Clone)]
pub struct CompensationAction {
    pub task_id: String,
    pub action: Action,
    pub condition: Condition,
}

impl CompensationAction {
    pub async fn execute(&self, context: &TaskContext) -> Result<(), CompensationError> {
        match self.action {
            Action::Rollback => self.rollback(context).await,
            Action::Cleanup => self.cleanup(context).await,
            Action::Notify => self.notify(context).await,
        }
    }
}
```

## 监控和告警

### 工作流监控
```rust
use galaxy_ops::workflow::{WorkflowMonitor, MetricCollector};

let monitor = WorkflowMonitor::new();

// 收集指标
let metrics = monitor.collect_metrics().await?;

// 设置告警规则
monitor.add_rule("high_failure_rate", "failure_rate > 0.1").await?;

// 发送告警
monitor.send_alert("high_failure_rate", "Workflow failure rate is high").await?;
```

### 性能指标
```rust
#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
    pub total_instances: u64,
    pub running_instances: u64,
    pub completed_instances: u64,
    pub failed_instances: u64,
    pub average_execution_time: Duration,
    pub max_execution_time: Duration,
    pub min_execution_time: Duration,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub throughput: f64, // instances per minute
}

## 测试策略

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_workflow_definition() {
        let workflow = Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test workflow".to_string()),
            processes: vec![],
            variables: HashMap::new(),
            triggers: vec![],
            metadata: WorkflowMetadata::default(),
        };

        assert_eq!(workflow.id, "test-workflow");
        assert_eq!(workflow.name, "Test Workflow");
    }

    #[tokio::test]
    async fn test_workflow_execution() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        
        let instance = engine
            .create_instance("test-workflow")
            .build();
        
        let execution_id = engine.start(instance).await.unwrap();
        
        // Wait for completion
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let status = engine.get_status(execution_id).await.unwrap();
        assert_eq!(status, InstanceStatus::Completed);
    }

    #[test]
    fn test_error_handling() {
        let workflow = Workflow {
            id: "error-test".to_string(),
            name: "Error Test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            processes: vec![Process {
                id: "main".to_string(),
                name: "Main Process".to_string(),
                type_: ProcessType::Sequential,
                tasks: vec![Task {
                    id: "failing-task".to_string(),
                    name: "Failing Task".to_string(),
                    type_: TaskType::Script,
                    executor: TaskExecutor::Script,
                    inputs: HashMap::new(),
                    outputs: HashMap::new(),
                    retry_policy: RetryPolicy::default(),
                    timeout: Duration::from_secs(30),
                    conditions: vec![],
                }],
                transitions: vec![],
                variables: HashMap::new(),
                error_handling: ErrorHandling {
                    strategy: ErrorStrategy::Retry,
                    max_retries: 3,
                    fallback_task: None,
                },
            }],
            variables: HashMap::new(),
            triggers: vec![],
            metadata: WorkflowMetadata::default(),
        };

        assert_eq!(workflow.processes[0].error_handling.max_retries, 3);
    }
}
```

### 集成测试
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_workflow() {
        let engine = WorkflowEngine::new();
        
        // Load workflow from YAML
        let workflow_yaml = r#"
            workflow:
              id: "integration-test"
              name: "Integration Test"
              version: "1.0.0"
              processes:
                - id: "main"
                  name: "Main Process"
                  type: "sequential"
                  tasks:
                    - id: "task1"
                      name: "Task 1"
                      type: "script"
                      executor: "bash"
                      inputs:
                        script: "echo 'Hello World'"
        "#;
        
        let workflow: Workflow = serde_yaml::from_str(workflow_yaml).unwrap();
        
        let instance = engine
            .create_instance("integration-test")
            .workflow(workflow)
            .build();
        
        let execution_id = engine.start(instance).await.unwrap();
        
        // Wait for completion
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let status = engine.get_status(execution_id).await.unwrap();
        assert_eq!(status, InstanceStatus::Completed);
    }
}
```

## 最佳实践

### 1. 工作流设计原则
- **单一职责**: 每个工作流应该专注于一个特定的业务目标
- **可重用性**: 设计可重用的工作流组件和模板
- **可观测性**: 确保工作流执行过程可监控和追踪
- **容错性**: 设计完善的错误处理和补偿机制
- **可扩展性**: 支持水平扩展和分布式执行

### 2. 性能优化
- **任务并行化**: 充分利用并行任务提高执行效率
- **资源池化**: 使用连接池和线程池优化资源使用
- **缓存策略**: 缓存频繁访问的数据和计算结果
- **异步执行**: 使用异步编程模型提高并发性能
- **批量处理**: 批量处理相似任务减少开销

### 3. 安全考虑
- **权限控制**: 基于角色的访问控制(RBAC)
- **数据加密**: 敏感数据加密存储和传输
- **审计日志**: 完整的操作审计和追踪
- **输入验证**: 严格的输入验证和清理
- **资源限制**: 设置资源使用上限防止滥用

### 4. 监控和告警
- **关键指标**: 监控工作流成功率、执行时间、错误率
- **实时告警**: 关键异常实时告警通知
- **趋势分析**: 长期趋势分析和容量规划
- **健康检查**: 定期健康检查和故障自愈
- **性能基准**: 建立性能基准和SLA指标

## 扩展性

### 1. 自定义任务类型
```rust
impl TaskExecutor for CustomTaskExecutor {
    async fn execute(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // 实现自定义任务逻辑
        match context.task_type {
            TaskType::Custom(name) => {
                match name.as_str() {
                    "database-migration" => self.execute_migration(context).await,
                    "security-scan" => self.execute_security_scan(context).await,
                    "performance-test" => self.execute_performance_test(context).await,
                    _ => Err(TaskError::UnsupportedTaskType),
                }
            }
            _ => Err(TaskError::UnsupportedTaskType),
        }
    }
}
```

### 2. 自定义触发器
```rust
impl Trigger for CustomTrigger {
    async fn check(&self, context: &TriggerContext) -> bool {
        // 实现自定义触发逻辑
        match self.trigger_type {
            TriggerType::Custom(name) => {
                match name.as_str() {
                    "git-tag" => self.check_git_tag(context).await,
                    "docker-image" => self.check_docker_image(context).await,
                    "database-change" => self.check_database_change(context).await,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
```

### 3. 自定义存储后端
```rust
impl StateStore for CustomStateStore {
    async fn save(&self, instance: &WorkflowInstance) -> Result<(), StateError> {
        // 实现自定义存储逻辑
        self.database.save(instance).await
    }

    async fn load(&self, id: &str) -> Result<WorkflowInstance, StateError> {
        // 实现自定义加载逻辑
        self.database.load(id).await
    }

    async fn delete(&self, id: &str) -> Result<(), StateError> {
        // 实现自定义删除逻辑
        self.database.delete(id).await
    }
}
```

## 相关模块

- **module**: 模块管理和依赖关系
- **package**: 包管理和版本控制
- **system**: 系统级配置和管理
- **storage**: 数据持久化和存储管理
- **service**: 业务服务和API接口

## 使用示例

### 1. 创建工作流
```bash
# 创建工作流定义文件
echo '
workflow:
  id: "deploy-app"
  name: "Deploy Application"
  version: "1.0.0"
  
  processes:
    - id: "deploy"
      name: "Deploy Process"
      type: "sequential"
      
      tasks:
        - id: "build"
          name: "Build Application"
          type: "script"
          executor: "bash"
          inputs:
            script: "make build"
        
        - id: "test"
          name: "Run Tests"
          type: "script"
          executor: "bash"
          inputs:
            script: "make test"
        
        - id: "deploy"
          name: "Deploy to Production"
          type: "kubernetes"
          executor: "kubectl"
          inputs:
            action: "apply"
            manifest: "deployment.yml"
' > deploy-app.yml
```

### 2. 执行工作流
```bash
# 使用CLI执行工作流
gops workflow run --file deploy-app.yml --var environment=production

# 监控执行状态
gops workflow status --id <execution-id>

# 查看执行日志
gops workflow logs --id <execution-id>
```

### 3. 程序化使用
```rust
use galaxy_ops::workflow::{WorkflowEngine, WorkflowDefinition};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = WorkflowEngine::new();
    
    // 加载工作流定义
    let definition = WorkflowDefinition::from_file("deploy-app.yml").await?;
    
    // 创建工作流实例
    let instance = engine
        .create_instance("deploy-app")
        .definition(definition)
        .variable("environment", "production")
        .build();
    
    // 启动工作流
    let execution_id = engine.start(instance).await?;
    
    // 等待完成
    loop {
        let status = engine.get_status(execution_id).await?;
        match status {
            InstanceStatus::Completed => {
                println!("Workflow completed successfully");
                break;
            }
            InstanceStatus::Failed => {
                println!("Workflow failed");
                break;
            }
            _ => {
                println!("Workflow status: {:?}", status);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    Ok(())
}
```