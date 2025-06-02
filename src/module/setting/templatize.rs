use std::path::Path;

use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use super::TemplatePath;

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct TemplateCustom {
    label_beg: String,
    label_end: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct TemplateConfig {
    origin: (String, String),
    target: (String, String),
}

impl From<TemplateCustom> for TemplateConfig {
    fn from(value: TemplateCustom) -> Self {
        Self {
            origin: (value.label_beg, value.label_end),
            target: ("{{".into(), "}}".into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct TemplateTargets {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    includes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    excludes: Vec<String>,
}

impl TemplateTargets {
    pub fn export_paths(&self, root: &Path) -> TemplatePath {
        let includes = self.includes().iter().map(|x| root.join(x)).collect();
        let excludes = self.excludes().iter().map(|x| root.join(x)).collect();
        TemplatePath::new(includes, excludes)
    }
}

impl TemplateConfig {
    pub fn example() -> Self {
        TemplateConfig {
            origin: ("[[".into(), "]]".into()),
            target: ("{{".into(), "}}".into()),
        }
    }
}

impl TemplateTargets {
    pub fn example() -> Self {
        Self {
            includes: vec![],
            excludes: vec!["README.md".to_string()],
        }
    }
}

impl TemplateCustom {
    pub fn example() -> Self {
        Self {
            label_beg: "[[".into(),
            label_end: "]]".into(),
        }
    }
}
