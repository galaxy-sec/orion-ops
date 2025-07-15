use derive_more::From;

use super::{CommentFmt, LabelCoverter, TplResult};
#[derive(Debug, Clone, From)]
pub enum CustTmplLabel {
    None,
    Setting(LabelCoverter),
}

impl CustTmplLabel {
    pub fn convert(&self, cfmt: &CommentFmt, code: String) -> TplResult<String> {
        match self {
            CustTmplLabel::None => Ok(code),
            CustTmplLabel::Setting(t) => t.convert(cfmt, code),
        }
    }
    pub fn restore(&self, code: String) -> TplResult<String> {
        match self {
            CustTmplLabel::None => Ok(code),
            CustTmplLabel::Setting(t) => t.restore(code),
        }
    }
}
