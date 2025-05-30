use std::path::PathBuf;

use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct Setting {
    localize_tpl: Templateization,
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

impl Setting {
    pub fn example() -> Self {
        Self {
            localize_tpl: Templateization {
                includes: vec![],
                excludes: vec!["README.md".to_string()],
            },
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
    use super::*;
    use serde_yaml;

    #[test]
    fn test_setting_serialization() {
        let setting = Setting {
            localize_tpl: Templateization {
                includes: vec![
                    "templates/**/*.html".to_string(),
                    "static/**/*.js".to_string(),
                ],
                excludes: vec![
                    "templates/secret/*".to_string(),
                    "static/vendor/*".to_string(),
                ],
            },
        };

        let yaml = serde_yaml::to_string(&setting).unwrap();
        let expected = r#"localize_tpl:
  includes:
  - templates/**/*.html
  - static/**/*.js
  excludes:
  - templates/secret/*
  - static/vendor/*
"#;
        assert_eq!(yaml, expected);
    }

    #[test]
    fn test_setting_deserialization() {
        let yaml = r#"---
localize_tpl:
  includes:
    - templates/**/*.html
  excludes:
    - templates/secret/*
"#;
        let setting: Setting = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(setting.localize_tpl.includes, vec!["templates/**/*.html"]);
        assert_eq!(setting.localize_tpl.excludes, vec!["templates/secret/*"]);
    }
}
