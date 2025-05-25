use std::path::Path;

use derive_getters::Getters;
use orion_error::{ErrorOwe, StructError, UvsConfFrom};

use crate::{error::SpecResult, task::OperationType, types::Persistable};

#[derive(Getters, Clone, Debug, PartialEq)]
pub struct BashAction {
    task: OperationType,
    code: String,
}

impl BashAction {
    pub fn new(task: OperationType, code: String) -> Self {
        Self { task, code }
    }
}

impl Persistable<BashAction> for BashAction {
    fn save_to(&self, path: &Path) -> SpecResult<()> {
        let path_file = path.join("setup.sh");
        std::fs::write(path_file, self.code.as_str()).owe_res()?;
        Ok(())
    }

    fn load_from(path: &Path) -> SpecResult<Self> {
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
