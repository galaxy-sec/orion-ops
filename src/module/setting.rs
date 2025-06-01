use std::path::PathBuf;

use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct Setting {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    localize: Option<LocalizeConf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct LocalizeConf {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    paths: Option<Templateization>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    template: Option<TemplateCustom>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct TemplateCustom {
    origin: (String, String),
    target: (String, String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct Templateization {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    includes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    excludes: Vec<String>,
}

impl Templateization {
    pub fn export_paths(&self, root: &PathBuf) -> TemplatePath {
        let includes = self.includes().iter().map(|x| root.join(x)).collect();
        let excludes = self.excludes().iter().map(|x| root.join(x)).collect();
        TemplatePath { includes, excludes }
    }
}

impl LocalizeConf {
    pub fn example() -> Self {
        Self {
            paths: Some(Templateization {
                includes: vec![],
                excludes: vec!["README.md".to_string()],
            }),
            template: Some(TemplateCustom {
                origin: ("[[".into(), "]]".into()),
                target: ("{{".into(), "}}".into()),
            }),
        }
    }
}

impl TemplateCustom {
    pub fn example() -> Self {
        TemplateCustom {
            origin: ("[[".into(), "]]".into()),
            target: ("{{".into(), "}}".into()),
        }
    }
}
impl Setting {
    pub fn example() -> Self {
        Self {
            localize: Some(LocalizeConf::example()),
        }
    }
}

#[derive(Default, Clone, Debug, Getters)]
pub struct TemplatePath {
    includes: Vec<PathBuf>,
    excludes: Vec<PathBuf>,
}

impl TemplatePath {
    pub fn exclude_mut(&mut self) -> &mut Vec<PathBuf> {
        &mut self.excludes
    }
    pub fn is_exclude(&self, dst: &PathBuf) -> bool {
        for exclude in &self.excludes {
            if dst.starts_with(exclude) {
                return true;
            }
        }
        false
    }
    pub fn is_include(&self, dst: &PathBuf) -> bool {
        for exclude in &self.includes {
            if dst.starts_with(exclude) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use crate::types::Configable;

    use super::*;
    use orion_error::TestAssert;
    use serde_yaml;

    #[test]
    fn test_setting_serialization() {
        let temp_dir = temp_dir();
        let save_path = temp_dir.join("setting.yml");
        let setting = Setting::example();
        setting.save_conf(&save_path).assert();
        println!("{}", std::fs::read_to_string(&save_path).unwrap());
        Setting::from_conf(&save_path).assert();
    }

    #[test]
    fn test_local_serialization() {
        let setting = LocalizeConf {
            paths: Some(Templateization {
                includes: vec![
                    "templates/**/*.html".to_string(),
                    "static/**/*.js".to_string(),
                ],
                excludes: vec![
                    "templates/secret/*".to_string(),
                    "static/vendor/*".to_string(),
                ],
            }),
            template: None,
        };

        let yaml = serde_yaml::to_string(&setting).assert();
        let expected = r#"paths:
  includes:
  - templates/**/*.html
  - static/**/*.js
  excludes:
  - templates/secret/*
  - static/vendor/*
"#;
        assert_eq!(yaml, expected);
        let _setting: LocalizeConf = serde_yaml::from_str(expected).assert();
    }
}
