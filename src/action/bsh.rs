use std::path::PathBuf;

use derive_getters::Getters;
use orion_error::{ErrorOwe, StructError, UvsConfFrom};

use crate::{error::SpecResult, task::OperationType, types::Persistable};

use super::HOST_SETUP_SH;

#[derive(Getters, Clone, Debug, PartialEq)]
pub struct BashAction {
    task: OperationType,
    code: String,
}

impl BashAction {
    pub fn setup_tpl() -> Self {
        Self {
            task: OperationType::Setup,
            code: HOST_SETUP_SH.to_string(),
        }
    }
}

impl Persistable<BashAction> for BashAction {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let path_file = path.join("setup.sh");
        std::fs::write(path_file, self.code.as_str()).owe_res()?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<Self> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad file name".to_string()))?;

        let task_type = match file_name {
            "setup.sh" => OperationType::Setup,
            "update.sh" => OperationType::Update,
            _ => todo!(),
        };
        let code = std::fs::read_to_string(path).owe_res()?;
        Ok(Self {
            task: task_type,
            code,
        })
    }
}
