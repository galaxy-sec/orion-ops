use derive_more::From;
use orion_error::{DomainReason, ErrorCode, StructError, StructErrorTrait, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum SpecReason {
    #[error("unknow")]
    UnKnow,
    #[error("localize:{0}")]
    Localize(LocalizeReason),
    #[error("element:{0}")]
    Element(ElementReason),
    #[error("{0}")]
    Uvs(UvsReason),
}

#[derive(Clone, Debug, Serialize, PartialEq, Error)]
pub enum ElementReason {
    #[error("miss:{0}")]
    Miss(String),
}
#[derive(Clone, Debug, Serialize, PartialEq, Error)]
pub enum LocalizeReason {
    #[error("miss:{0}")]
    Templatize(String),
}
impl ErrorCode for ElementReason {
    fn error_code(&self) -> i32 {
        match self {
            ElementReason::Miss(_) => 531,
        }
    }
}

impl ErrorCode for LocalizeReason {
    fn error_code(&self) -> i32 {
        match self {
            LocalizeReason::Templatize(_) => 541,
        }
    }
}

impl ErrorCode for SpecReason {
    fn error_code(&self) -> i32 {
        match self {
            SpecReason::UnKnow => 500,
            SpecReason::Uvs(r) => r.error_code(),
            SpecReason::Localize(r) => r.error_code(),
            SpecReason::Element(r) => r.error_code(),
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

pub const PATH_NOT_EXIST: &str = "path not exists";

pub fn report_error(e: StructError<SpecReason>) {
    println!("Galaxy Flow Parse Error (Code: {})", e.error_code());
    println!("--------------------------");
    if let Some(target) = e.target() {
        println!("[TARGET]:\n{}\n", target);
    }
    println!("[REASON]:");
    match e.get_reason() {
        SpecReason::Uvs(uvs_reason) => match uvs_reason {
            UvsReason::LogicError(e) => {
                println!("LOGIC ERROR: {}\n", e);
            }
            UvsReason::BizError(e) => {
                println!("BIZ ERROR: {}\n", e);
            }
            UvsReason::DataError(e, _) => {
                println!("DATA ERROR: {}\n", e);
            }
            UvsReason::SysError(e) => {
                println!("SYS ERROR: {}\n", e);
            }
            UvsReason::ResError(e) => {
                println!("RES ERROR: {}\n", e);
            }
            UvsReason::ConfError(e) => {
                println!("CONF ERROR: {}\n", e);
            }
            UvsReason::RuleError(e) => {
                println!("RULE ERROR: {}\n", e);
            }
            UvsReason::PrivacyError(e) => {
                println!("PRIVACY ERROR: {}\n", e);
            }
        },

        SpecReason::Localize(e) => {
            println!("Localize ERROR: {}\n", e);
        }
        SpecReason::Element(e) => {
            println!("Element ERROR: {}\n", e);
        }
        SpecReason::UnKnow => {
            println!("Unknow Error!\n");
        }
    }
    if let Some(pos) = e.position() {
        println!("\n[POSITION]:\n{}", pos);
    }
    if let Some(detail) = e.detail() {
        println!("\n[DETAIL]:\n{}", detail);
    }
    println!("\n[CONTEXT]:\n");
    for x in e.context() {
        println!("{}", x)
    }
}
