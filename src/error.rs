use orion_error::{DomainReason, ErrorCode, StructError};
use serde_derive::Serialize;
use std::fmt::Display;
#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum RunReason {
    UnKnow,
    What(String),
}

impl Display for RunReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunReason::UnKnow => todo!(),
            RunReason::What(msg) => {
                write!(f, "{}", msg)?;
            }
        }
        Ok(())
    }
}
impl DomainReason for RunReason {}
impl ErrorCode for RunReason {
    fn error_code(&self) -> i32 {
        500
    }
}

pub type RunResult<T> = Result<T, StructError<RunReason>>;
