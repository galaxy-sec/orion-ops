use std::path::PathBuf;

use derive_getters::Getters;
use orion_error::{ErrorOwe, StructError, UvsConfFrom};

use crate::{error::SpecResult, task::OperationType, types::Persistable};

use super::{HOST_SETUP_GXL, HOST_UPDATE_GXL, K8S_SETUP_GXL, K8S_UPDATE_GXL};

#[derive(Getters, Clone, Debug, PartialEq)]
pub struct GxlAction {
    task: OperationType,
    code: String,
}

impl GxlAction {
    pub fn new(task: OperationType, code: String) -> Self {
        Self { task, code }
    }
    pub fn host_setup_tpl() -> Self {
        Self {
            task: OperationType::Setup,
            code: HOST_SETUP_GXL.to_string(),
        }
    }
    pub fn host_update_tpl() -> Self {
        Self {
            task: OperationType::Update,
            code: HOST_UPDATE_GXL.to_string(),
        }
    }
    pub fn k8s_setup_tpl() -> Self {
        Self {
            task: OperationType::Setup,
            code: K8S_SETUP_GXL.to_string(),
        }
    }
    pub fn k8s_update_tpl() -> Self {
        Self {
            task: OperationType::Update,
            code: K8S_UPDATE_GXL.to_string(),
        }
    }
}
impl Persistable<GxlAction> for GxlAction {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let path_file = path.join(format!("{}.gxl", self.task().to_string()));
        std::fs::write(path_file, self.code.as_str()).owe_res()?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<GxlAction> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad file name".to_string()))?;

        let task_type = match file_name {
            "setup.gxl" => OperationType::Setup,
            "update.gxl" => OperationType::Update,
            _ => todo!(),
        };
        let code = std::fs::read_to_string(path).owe_res()?;
        Ok(Self {
            task: task_type,
            code,
        })
    }
}
