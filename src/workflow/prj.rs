use std::path::Path;

use derive_getters::Getters;
use orion_common::serde::{Persistable, SerdeResult};
use orion_error::{ErrorOwe, ErrorWith};
use serde::Serialize;

use crate::const_vars::{ADM_GXL, PRJ_TOML};

#[derive(Getters, Clone, Debug, Default, Serialize)]
pub struct GxlProject {
    work: String,
    adm: Option<String>,
    prj: Option<String>,
}
impl From<&str> for GxlProject {
    fn from(value: &str) -> Self {
        Self {
            work: value.to_string(),
            adm: None,
            prj: None,
        }
    }
}

impl From<(&str, &str)> for GxlProject {
    fn from(value: (&str, &str)) -> Self {
        Self {
            work: value.0.to_string(),
            adm: Some(value.1.to_string()),
            prj: None,
        }
    }
}

impl From<(&str, &str, &str)> for GxlProject {
    fn from(value: (&str, &str, &str)) -> Self {
        Self {
            work: value.0.to_string(),
            adm: Some(value.1.to_string()),
            prj: Some(value.2.to_string()),
        }
    }
}

impl Persistable<GxlProject> for GxlProject {
    fn save_to(&self, path: &Path, _name: Option<String>) -> SerdeResult<()> {
        let gal_path = path.join("_gal");
        std::fs::create_dir_all(&gal_path)
            .owe_res()
            .with(&gal_path)?;
        std::fs::write(
            gal_path.join(crate::const_vars::WORK_GXL),
            self.work.as_str(),
        )
        .owe_res()?;
        if let Some(adm) = &self.adm {
            std::fs::write(gal_path.join(ADM_GXL), adm.as_str()).owe_res()?;
            let version_path = path.join("version.txt");
            if !version_path.exists() {
                std::fs::write(version_path, "0.1.0")
                    .owe_res()
                    .want("crate version.txt")?;
            }
        }
        if let Some(prj) = &self.prj {
            std::fs::write(gal_path.join(PRJ_TOML), prj.as_str()).owe_res()?;
        }
        Ok(())
    }

    fn load_from(path: &Path) -> SerdeResult<GxlProject> {
        let work_path = path.join("_gal/work.gxl");
        let adm_path = path.join("_gal/project.toml");
        let prj_path = path.join("_gal/project.toml");
        let work = std::fs::read_to_string(work_path).owe_res()?;
        let adm = if adm_path.exists() {
            Some(std::fs::read_to_string(adm_path).owe_res()?)
        } else {
            None
        };
        let prj = if prj_path.exists() {
            Some(std::fs::read_to_string(prj_path).owe_res()?)
        } else {
            None
        };
        Ok(Self { work, adm, prj })
    }
}
