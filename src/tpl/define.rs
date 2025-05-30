use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::{
    error::SpecResult,
    module::setting::TemplatePath,
    tpl::{gtmpl::TplGtmpl, handlebars::TplHandleBars},
};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TPlEngineType {
    #[default]
    Handlebars,
    GTmpl,
}
impl FromStr for TPlEngineType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "handlebars" => Ok(Self::Handlebars),
            "gtmpl" => Ok(Self::GTmpl),
            _ => Err("unknow engine".to_string()),
        }
    }
}
impl Display for TPlEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TPlEngineType::Handlebars => "handlebars",
            TPlEngineType::GTmpl => "helm",
        };
        write!(f, "{}", msg)
    }
}
pub struct TplRender;
impl TplRender {
    pub fn render_path(
        engine: TPlEngineType,
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &PathBuf,
        setting: &TemplatePath,
    ) -> SpecResult<()> {
        match engine {
            TPlEngineType::Handlebars => TplHandleBars::render_path(tpl, dst, data, setting),
            TPlEngineType::GTmpl => TplGtmpl::render_path(tpl, dst, data, setting),
        }
    }
}
