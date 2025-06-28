use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{VERSION, mcp_protocol::*};
use actix_web::{
     HttpResponse,  Responder,  get, post,
    web::{self, Data},
};
use anyhow::{Result, anyhow};
use serde_json::json;
use tracing::info;

// 定义服务状态
#[derive(Clone)]
pub struct AppState {
    request_counter: Arc<AtomicU64>, // 原子计数器
}

impl AppState {
    // 创建新实例
    pub fn new() -> Self {
        AppState {
            request_counter: Arc::new(AtomicU64::new(0)),
        }
    }
    // ... 已有的 add、get_system_info、process_text 方法 ...

    // 新增：处理 initialize 请求
    async fn initialize(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        // 解析客户端信息（可选）
        let manifest =  build_manifest() ;

        Ok(json!({
            "jsonrpc": "2.0",
            "result": serde_json::to_value(manifest)?,
            "id": params.get("id").cloned().unwrap_or(serde_json::Value::Null)
        }))
    }

    // 示例能力：计算两个数之和
    async fn add(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let a = params["a"]
            .as_f64()
            .ok_or_else(|| anyhow!("参数 'a' 缺失或类型错误"))?;
        let b = params["b"]
            .as_f64()
            .ok_or_else(|| anyhow!("参数 'b' 缺失或类型错误"))?;

        Ok(json!({ "result": a + b }))
    }

    // 示例能力：获取系统信息
    async fn get_system_info(&self) -> Result<serde_json::Value> {
        let count = self.request_counter.fetch_add(1, Ordering::SeqCst);

        Ok(json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "request_count": count,
            "version": VERSION.as_str()
        }))
    }

    // 示例能力：文本处理
    async fn process_text(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let text = params["text"]
            .as_str()
            .ok_or_else(|| anyhow!("参数 'text' 缺失或类型错误"))?;

        // 这里可以添加复杂的文本处理逻辑
        let processed = text.to_uppercase();

        Ok(json!({ "processed_text": processed }))
    }
}

// 处理 MCP 请求
#[post("/mcp")]
pub async fn handle_mcp_request(
    req: web::Json<MCPRequest>,
    state: Data<AppState>,
) -> impl Responder {
    // 记录日志
    info!("Received MCP request: {:?}", req);

    // 处理请求
    let result = match req.method.as_str() {
        "add" => state.add(req.params.as_ref().unwrap_or(&json!({}))).await,
        "getSystemInfo" => state.get_system_info().await,
        "processText" => {
            state
                .process_text(req.params.as_ref().unwrap_or(&json!({})))
                .await
        }
        "initialize" => {
            state
                .initialize(req.params.as_ref().unwrap_or(&json!({})))
                .await
        }
        _ => Err(anyhow!("未知的方法: {}", req.method)),
    };

    // 构建响应
    match result {
        Ok(result) => HttpResponse::Ok().json(MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(result),
            error: None,
        }),
        Err(err) => {
            tracing::error!("处理请求失败: {}", err);
            HttpResponse::Ok().json(MCPResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result: None,
                error: Some(MCPError {
                    code: 500,
                    message: err.to_string(),
                    data: None,
                }),
            })
        }
    }
}

pub fn build_manifest() -> Manifest {
    Manifest {
    protocol_version: "2.0".to_string(),
    /* 
    capabilities: vec![
        Capability {
            name: "add".to_string(),
            description: "Add two numbers".to_string(),
            parameters: serde_json::json!({})
        }
    ],
    */
    server_info: ServerInfo {
        name: "ds-mcp".to_string(),
        version: "1.0.0".to_string(),
        description: "Data Science MCP Service".to_string()
    },
        name: "AI能力服务".to_string(),
        description: "通过MCP协议提供多种AI能力的服务".to_string(),
        version: VERSION.to_string(),
        capabilities: vec![
            Capability {
                name: "add".to_string(),
                description: "计算两个数的和".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "第一个数字"},
                        "b": {"type": "number", "description": "第二个数字"}
                    },
                    "required": ["a", "b"]
                }),
            },
            Capability {
                name: "getSystemInfo".to_string(),
                description: "获取系统信息".to_string(),
                parameters: json!({}),
            },
            Capability {
                name: "processText".to_string(),
                description: "处理文本（示例：转换为大写）".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "需要处理的文本"}
                    },
                    "required": ["text"]
                }),
            },
        ],
    }
}

// 提供服务清单
#[get("/manifest")]
pub async fn get_manifest() -> impl Responder {

    HttpResponse::Ok().json(build_manifest())
}
