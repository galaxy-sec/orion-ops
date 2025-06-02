use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use super::{TemplateCustom, TemplateTargets};

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct LocalizeConf {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    templatize_path: Option<TemplateTargets>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    templatize_cust: Option<TemplateCustom>,
}

impl LocalizeConf {
    pub fn example() -> Self {
        Self {
            templatize_path: Some(TemplateTargets::example()),
            templatize_cust: Some(TemplateCustom::example()),
        }
    }
}
