use std::path::Path;

use derive_getters::Getters;
use orion_error::{ErrorOwe, StructError, UvsConfFrom};
use orion_x::saveable::{Persistable, SerdeResult};
use serde::Serialize;

use crate::task::OperationType;

#[derive(Getters, Clone, Debug, PartialEq, Serialize)]
pub struct GxlAction {
    task: OperationType,
    file: String,
    code: String,
}

impl GxlAction {
    pub fn new(task: OperationType, file: String, code: String) -> Self {
        Self { task, file, code }
    }
    pub fn is_action(path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            return matches!(
                file_name,
                "setup.gxl" | "update.gxl" | "port.gxl" | "backup.gxl" | "uninstall.gxl"
            );
        }
        false
    }
}
impl Persistable<GxlAction> for GxlAction {
    fn save_to(&self, path: &Path, _name: Option<String>) -> SerdeResult<()> {
        let path_file = path.join(self.file());
        std::fs::write(path_file, self.code.as_str()).owe_res()?;
        Ok(())
    }

    fn load_from(path: &Path) -> SerdeResult<GxlAction> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad file name".to_string()))?;

        let task_type = match file_name {
            "setup.gxl" => OperationType::Setup,
            "update.gxl" => OperationType::Update,
            "port.gxl" => OperationType::Port,
            "backup.gxl" => OperationType::Backup,
            "uninstall.gxl" => OperationType::UnInstall,
            _ => OperationType::Other,
        };
        let code = std::fs::read_to_string(path).owe_res()?;
        Ok(Self {
            task: task_type,
            file: file_name.to_string(),
            code,
        })
    }
}
