use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum SpecReason {
    #[error("unknow")]
    UnKnow,
    #[error("{0}")]
    What(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for SpecReason {
    fn error_code(&self) -> i32 {
        match self {
            SpecReason::UnKnow => 500,
            SpecReason::What(_) => 501,
            SpecReason::Uvs(uvs_reason) => uvs_reason.error_code(),
        }
    }
}

pub type SpecResult<T> = Result<T, StructError<SpecReason>>;
