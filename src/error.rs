use derive_more::From;
use orion_error::{DomainReason, ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum SpecReason {
    #[error("unknow")]
    UnKnow,
    #[error("miss:{0}")]
    Miss(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for SpecReason {
    fn error_code(&self) -> i32 {
        match self {
            SpecReason::UnKnow => 500,
            SpecReason::Miss(_) => 501,
            SpecReason::Uvs(uvs_reason) => uvs_reason.error_code(),
        }
    }
}

pub trait ToErr<R>
where
    R: DomainReason,
{
    fn to_err(self) -> StructError<R>;
    fn err_result<T>(self) -> Result<T, StructError<R>>;
}
impl<R> ToErr<R> for R
where
    R: DomainReason,
{
    fn to_err(self) -> StructError<R> {
        StructError::from(self)
    }
    fn err_result<T>(self) -> Result<T, StructError<R>> {
        Err(StructError::from(self))
    }
}

pub type SpecResult<T> = Result<T, StructError<SpecReason>>;
