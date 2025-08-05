use galaxy_ops::error::SpecError;
use rmcp::model::{ErrorCode, ErrorData};

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error("Operation failed with ErrorA: {0}")]
    Mcp(#[from] ErrorData), // 自动从 ErrorA 转换（依赖 From 实现）
    #[error("Operation failed with ErrorB: {0}")]
    Spec(#[from] SpecError), // 自动从 ErrorB 转换
}

impl From<AppError> for ErrorData {
    fn from(value: AppError) -> Self {
        match value {
            AppError::Mcp(error_data) => error_data,
            AppError::Spec(struct_error) => ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                "spec service error",
                Some(Value::String(struct_error.to_string())),
            ),
        }
    }
}

pub trait ConvMcpError<T> {
    fn mcp_err(self) -> Result<T, ErrorData>;
}

impl<T> ConvMcpError<T> for Result<T, SpecError> {
    fn mcp_err(self) -> Result<T, ErrorData> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                "spec service error",
                Some(Value::String(e.to_string())),
            )),
        }
    }
}
