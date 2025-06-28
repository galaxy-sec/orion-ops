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
use log::debug;
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

#[post("/mcp")]
pub async fn handle_mcp_request(
    req: web::Json<MCPRequest>,
    state: Data<AppState>,
) -> impl Responder {
    info!("Received MCP request: {:?}", req);

    // 初始化 MCPResponse，包含 jsonrpc 和请求 id
    let mut response = MCPResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id.clone(), // 保留客户端请求的 id（可能为 null）
        result: None,
        error: None,
    };

    // 根据方法名处理请求，并填充 result 或 error
    match req.method.as_str() {
        "add" => {
            let params = req.params.as_ref().unwrap_or(&json!(null));
            match state.add(params).await {
                Ok(result) => response.result = Some(result),
                Err(err) => {
                    response.error = Some(MCPError {
                        code: 500,
                        message: err.to_string(),
                        data: None,
                    });
                }
            }
        }
        "getSystemInfo" => {
            match state.get_system_info().await {
                Ok(result) => response.result = Some(result),
                Err(err) => {
                    response.error = Some(MCPError {
                        code: 500,
                        message: err.to_string(),
                        data: None,
                    });
                }
            }
        }
        "processText" => {
            let params = req.params.as_ref().unwrap_or(&json!(null));
            match state.process_text(params).await {
                Ok(result) => response.result = Some(result),
                Err(err) => {
                    response.error = Some(MCPError {
                        code: 500,
                        message: err.to_string(),
                        data: None,
                    });
                }
            }
        }
        "initialize" => {
            let params = req.params.as_ref().unwrap_or(&json!(null));
            match state.initialize(params).await {
                Ok(result) => response.result = Some(result),
                Err(err) => {
                    response.error = Some(MCPError {
                        code: 500,
                        message: err.to_string(),
                        data: None,
                    });
                }
            }
        }
        _ => {
            response.error = Some(MCPError {
                code: -32601, // JSON-RPC 标准方法未找到错误码
                message: format!("Method '{}' not found", req.method),
                data: None,
            });
        }
    }

    // 返回统一包装后的 MCPResponse
    HttpResponse::Ok().json(response)
}

pub fn build_manifest() -> Manifest {
    Manifest {
        protocol_version: "1.0.0".to_string(), // 根据协议规范设置版本
        capabilities: serde_json::json!( {
            "tool_invocation": { 
                "description": "支持工具调用",
                "max_concurrent": 5 
            },
            "resource_management": {
                "description": "支持资源读写",
                "max_size": 1024 * 1024 // 1MB
            }
        }),
        server_info: ServerInfo {
            name: "ds-mcp".to_string(),
            version: "1.0.0".to_string(),
            description: "Data Science MCP Service".to_string(),
        },
    }
}

// 提供服务清单
#[get("/manifest")]
pub async fn get_manifest() -> impl Responder {

    HttpResponse::Ok().json(build_manifest())
}
