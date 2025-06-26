use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{VERSION, mcp_protocol::*};
use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, error, get, post,
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
        let client_info = params
            .get("clientInfo")
            .ok_or_else(|| anyhow!("缺少 clientInfo 参数"))?;
        let client_name = client_info
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("未知客户端");
        let client_version = client_info
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("未知版本");

        // 解析协议版本（可选）
        let protocol_version = params
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("未知协议版本");

        // 返回初始化成功的信息（可根据需求扩展）
        Ok(json!(
            {
                "status": "success",
                "message": format!("欢迎使用 {} (版本: {})，服务端协议版本: {}", client_name, client_version, protocol_version),
                "supported_methods": ["add", "getSystemInfo", "processText", "initialize"], // 列出所有支持的方法
                "server_time": chrono::Utc::now().to_rfc3339() // 可选：返回服务器时间
            }
        ))
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

// 提供服务清单
#[get("/manifest")]
pub async fn get_manifest() -> impl Responder {
    let manifest = Manifest {
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
    };

    HttpResponse::Ok().json(manifest)
}
