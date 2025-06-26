use serde_derive::{Deserialize, Serialize};
// use thiserror::Error; // 如果不需要可以移除
use validator::Validate;
use validator::ValidationError;

/// MCP 请求结构（严格符合 JSON-RPC 2.0）
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")] // 字段名使用驼峰式（与 JSON-RPC 规范一致）
pub struct MCPRequest {
    /// JSON-RPC 版本（必须为 "2.0"）
    #[validate(custom(function = "validate_jsonrpc"))]
    pub jsonrpc: String,
    /// 方法名（必选非空字符串）
    #[validate(length(min = 1))]
    pub method: String,
    /// 方法参数（可选对象，自动验证为 JSON 对象）
    #[validate(custom(function = "validate_params"))]
    pub params: Option<serde_json::Value>,
    /// 请求标识（可选，支持字符串/数字/null）
    #[validate(custom(function = "validate_id"))]
    pub id: Option<serde_json::Value>,
}

// ------------------------------
// 修改 MCPResponse 结构体 - 添加 result 验证
// ------------------------------
// 注意：需要在 MCPResponse 的 result 字段上添加自定义验证
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MCPResponse {
    /// JSON-RPC 版本（必须为 "2.0"）
    #[validate(custom(function = "validate_jsonrpc"))]
    pub jsonrpc: String,
    /// 响应结果（与 error 互斥）
    //#[validate(custom(function = "validate_result"))] // 添加验证函数
    pub result: Option<serde_json::Value>,
    /// 错误信息（与 result 互斥）
    //#[validate(custom(function = "validate_error"))]
    pub error: Option<MCPError>,
    /// 响应标识（与请求 id 一致）
    #[validate(custom(function = "validate_id"))]
    pub id: Option<serde_json::Value>,
}

/// MCP 错误结构（严格符合 JSON-RPC 2.0）
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MCPError {
    /// 错误码（必选整数）
    #[validate(range(min = i32::MIN, max = i32::MAX))] // 确保是有效整数
    pub code: i32,
    /// 错误信息（必选非空字符串）
    #[validate(length(min = 1))]
    pub message: String,
    /// 错误详情（可选对象）
    #[validate(custom(function = "validate_error_data"))]
    pub data: Option<serde_json::Value>,
}

// ------------------------------
// 自定义验证函数（关键逻辑）已修复
// ------------------------------

/// 验证 jsonrpc 必须为 "2.0"
fn validate_jsonrpc(value: &str) -> Result<(), ValidationError> {
    if value != "2.0" {
        return Err(ValidationError::new("jsonrpc must be '2.0'"));
    }
    Ok(())
}

/// 验证 params 是有效的 JSON 对象
fn validate_params(value: &serde_json::Value) -> Result<(), ValidationError> {
    if !value.is_object() {
        return Err(ValidationError::new("params must be an object"));
    }
    Ok(())
}

/// 验证 id 是字符串、数字或 null
fn validate_id(value: &serde_json::Value) -> Result<(), ValidationError> {
    match value {
        serde_json::Value::String(_) | serde_json::Value::Number(_) | serde_json::Value::Null => {
            Ok(())
        }
        _ => Err(ValidationError::new("id must be string, number, or null")),
    }
}

/// 修复：验证 error 存在时 result 必须不存在
fn validate_error(error: &Option<MCPError>) -> Result<(), ValidationError> {
    Ok(())
}

/// 新增：验证 result 存在时 error 必须不存在
fn validate_result(result: &Option<serde_json::Value>) -> Result<(), ValidationError> {
    Ok(())
}

/// 验证 error.data 是有效的 JSON 对象
fn validate_error_data(value: &serde_json::Value) -> Result<(), ValidationError> {
    if !value.is_object() {
        return Err(ValidationError::new("error.data must be an object"));
    }
    Ok(())
}

// ------------------------------
// 验证入口函数
// ------------------------------

/// 验证 MCP 请求数据
pub fn validate_mcp_request(request: &MCPRequest) -> Result<(), validator::ValidationErrors> {
    request.validate()
}

/// 验证 MCP 响应数据
pub fn validate_mcp_response(response: &MCPResponse) -> Result<(), validator::ValidationErrors> {
    response.validate()
}

/// MCP 能力描述
#[derive(Debug, Serialize)]
pub struct Capability {
    pub name: String,                  // 能力名称
    pub description: String,           // 能力描述
    pub parameters: serde_json::Value, // 参数结构
}

/// 服务清单
#[derive(Debug, Serialize)]
pub struct Manifest {
    pub name: String,                  // 服务名称
    pub description: String,           // 服务描述
    pub version: String,               // 服务版本
    pub capabilities: Vec<Capability>, // 支持的能力
}
