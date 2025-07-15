use std::ffi::OsStr;

use rust::CStyleComment;
use yml::YmlComment;

use super::TplResult;

mod rust;
mod yml;

#[derive(Debug, Clone)]
pub enum CommentFmt {
    CStyle,
    Yml,
    UnNeed,
}

impl CommentFmt {
    pub fn remove(&self, code: &str) -> TplResult<String> {
        match self {
            CommentFmt::CStyle => CStyleComment::remove(code),
            CommentFmt::Yml => YmlComment::remove(code),
            CommentFmt::UnNeed => Ok(code.to_string()),
        }
    }
}

impl From<Option<&OsStr>> for CommentFmt {
    fn from(value: Option<&OsStr>) -> Self {
        match value.and_then(|x| x.to_str()) {
            Some("yml") => Self::Yml,
            Some("yaml") => Self::Yml,
            Some(".c") => Self::CStyle,
            Some(".cpp") => Self::CStyle,
            _ => Self::UnNeed,
        }
    }
}
