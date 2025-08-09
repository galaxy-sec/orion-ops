use derive_more::From;
use orion_error::{DomainReason, ErrorCode, StructError, StructErrorTrait, UvsReason, UvsResFrom};
use orion_variate::addr::AddrReason;
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum MainReason {
    #[error("unknow")]
    UnKnow,
    #[error("localize:{0}")]
    Localize(LocalizeReason),
    #[error("element:{0}")]
    Element(ElementReason),
    #[error("mod {0}")]
    Mod(ModReason),
    #[error("sys {0}")]
    Sys(SysReason),
    #[error("sys {0}")]
    Ops(OpsReason),
    #[error("{0}")]
    Uvs(UvsReason),
}

#[derive(Clone, Debug, Serialize, PartialEq, Error)]
pub enum ElementReason {
    #[error("miss:{0}")]
    Miss(String),
}
#[derive(Clone, Debug, Serialize, PartialEq, Error)]
pub enum ModReason {
    #[error("miss:{0}")]
    Miss(String),
    #[error("load fail")]
    Load,
    #[error("save fail")]
    Save,
    #[error("update fail")]
    Update,
    #[error("localize fail")]
    Localize,
}
#[derive(Clone, Debug, Serialize, PartialEq, Error)]
pub enum SysReason {
    #[error("miss:{0}")]
    Miss(String),
    #[error("load fail")]
    Load,
    #[error("save fail")]
    Save,
    #[error("update fail")]
    Update,
    #[error("localize fail")]
    Localize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Error)]
pub enum OpsReason {
    #[error("miss:{0}")]
    Miss(String),
    #[error("load fail")]
    Load,
    #[error("save fail")]
    Save,
    #[error("update fail")]
    Update,
    #[error("localize fail")]
    Localize,
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
impl ErrorCode for ModReason {
    fn error_code(&self) -> i32 {
        match self {
            Self::Miss(_) => 551,
            ModReason::Load => 552,
            ModReason::Save => 553,
            ModReason::Update => 554,
            ModReason::Localize => 555,
        }
    }
}
impl ErrorCode for SysReason {
    fn error_code(&self) -> i32 {
        match self {
            SysReason::Miss(_) => 561,
            SysReason::Load => 562,
            SysReason::Save => 563,
            SysReason::Update => 564,
            SysReason::Localize => 565,
        }
    }
}

impl ErrorCode for OpsReason {
    fn error_code(&self) -> i32 {
        match self {
            OpsReason::Miss(_) => 571,
            OpsReason::Load => 572,
            OpsReason::Save => 573,
            OpsReason::Update => 574,
            OpsReason::Localize => 575,
        }
    }
}

impl ErrorCode for MainReason {
    fn error_code(&self) -> i32 {
        match self {
            MainReason::UnKnow => 500,
            MainReason::Uvs(r) => r.error_code(),
            MainReason::Localize(r) => r.error_code(),
            MainReason::Element(r) => r.error_code(),
            MainReason::Mod(r) => r.error_code(),
            MainReason::Sys(r) => r.error_code(),
            MainReason::Ops(r) => r.error_code(),
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

impl From<AddrReason> for MainReason {
    fn from(value: AddrReason) -> Self {
        match value {
            AddrReason::Brief(msg) => Self::Uvs(UvsReason::from_res(msg)),
            AddrReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            AddrReason::OperationTimeoutExceeded { timeout, attempts } => {
                Self::Uvs(UvsReason::from_res(format!(
                    "timeout:{}s attempts: {attempts}",
                    timeout.as_secs()
                )))
            }
            AddrReason::TotalTimeoutExceeded {
                total_timeout,
                elapsed,
            } => Self::Uvs(UvsReason::from_res(format!(
                "timeout:{}s elapsed: {}",
                total_timeout.as_secs(),
                elapsed.as_secs()
            ))),
            AddrReason::RetryExhausted {
                attempts,
                last_error,
            } => Self::Uvs(UvsReason::from_res(format!(
                "attempts:{attempts} last_error: {last_error}",
            ))),
        }
    }
}
pub type MainResult<T> = Result<T, StructError<MainReason>>;
pub type MainError = StructError<MainReason>;

pub const PATH_NOT_EXIST: &str = "path not exists";

pub fn report_error(e: StructError<MainReason>) {
    println!("Run Error (Code: {})", e.error_code());
    println!("--------------------------");
    if let Some(target) = e.target() {
        println!("[TARGET]:\n{target}\n",);
    }
    println!("[REASON]:");
    match e.get_reason() {
        MainReason::Uvs(uvs_reason) => match uvs_reason {
            UvsReason::LogicError(e) => {
                println!("LOGIC ERROR: {e}\n",);
            }
            UvsReason::BizError(e) => {
                println!("BIZ ERROR: {e}\n",);
            }
            UvsReason::DataError(e, _) => {
                println!("DATA ERROR: {e}\n",);
            }
            UvsReason::SysError(e) => {
                println!("SYS ERROR: {e}\n",);
            }
            UvsReason::ResError(e) => {
                println!("RES ERROR: {e}\n",);
            }
            UvsReason::ConfError(e) => {
                println!("CONF ERROR: {e}\n",);
            }
            UvsReason::RuleError(e) => {
                println!("RULE ERROR: {e}\n",);
            }
            UvsReason::PrivacyError(e) => {
                println!("PRIVACY ERROR: {e}\n",);
            }
        },

        MainReason::Localize(e) => {
            println!("Localize ERROR: {e}\n",);
        }
        MainReason::Element(e) => {
            println!("Element ERROR: {e}\n",);
        }
        MainReason::UnKnow => {
            println!("Unknow Error!\n");
        }
        MainReason::Mod(e) => {
            println!("Mod Error: \n{e} !");
        }
        MainReason::Sys(e) => {
            println!("Sys Error: \n{e}");
        }
        MainReason::Ops(e) => {
            println!("Operator Error: \n{e}");
        }
    }
    if let Some(pos) = e.position() {
        println!("\n[POSITION]:\n{pos}",);
    }
    if let Some(detail) = e.detail() {
        println!("\n[DETAIL]:\n{detail}",);
    }
    println!("\n[CONTEXT]:\n");
    for x in e.context() {
        println!("{x}",)
    }
}
