use std::path::Path;

use super::prj::GxlProject;
use super::{bsh::BashAction, gxl::GxlAction};
use derive_getters::Getters;
use log::warn;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom};

use crate::{error::SpecResult, types::Persistable};
#[derive(Getters, Clone, Debug)]
pub struct Actions {
    project: GxlProject,
    actions: Vec<ActionType>,
}

impl Actions {
    pub fn new(project: GxlProject, actions: Vec<ActionType>) -> Self {
        Self { project, actions }
    }
}

impl Persistable<Actions> for Actions {
    fn save_to(&self, path: &Path) -> SpecResult<()> {
        let action_path = path.join("actions");
        std::fs::create_dir_all(&action_path)
            .owe_res()
            .with(&action_path)?;
        for item in &self.actions {
            item.save_to(&action_path)?;
        }
        self.project.save_to(path)?;
        Ok(())
    }

    //加载 path 目录的文件
    fn load_from(path: &Path) -> SpecResult<Actions> {
        let mut actions = Vec::new();
        let actions_path = path.join("actions");
        for entry in std::fs::read_dir(&actions_path).owe_res()? {
            let entry = entry.owe_res()?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                let action = ActionType::load_from(&entry_path);
                match action {
                    Ok(act) => {
                        actions.push(act);
                    }
                    Err(e) => {
                        warn!("load ignore : {}\n {}", entry_path.display(), e);
                    }
                }
            }
        }
        let project = GxlProject::load_from(path).with(path)?;
        Ok(Actions { project, actions })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    Bash(BashAction),
    Gxl(GxlAction),
}

impl Persistable<ActionType> for ActionType {
    fn save_to(&self, path: &Path) -> SpecResult<()> {
        match self {
            ActionType::Bash(act) => act.save_to(path),
            ActionType::Gxl(act) => act.save_to(path),
        }
    }

    fn load_from(path: &Path) -> SpecResult<ActionType> {
        // 首先检查文件是否存在且是普通文件
        if !path.exists() {
            return Err(StructError::from_conf("path not exists".into())).with(path);
        }

        if !path.is_file() {
            return Err(StructError::from_conf("path not file".into())).with(path);
        }

        // 根据扩展名分发加载逻辑
        match path.extension().and_then(|s| s.to_str()) {
            Some("sh") => BashAction::load_from(path).map(ActionType::Bash),
            Some("gxl") => GxlAction::load_from(path).map(ActionType::Gxl),
            _ => Err(StructError::from_conf("file type not support".into())).with(path),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module::init::ModIniter;

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_host_tpl_init() {
        let actions = Actions::mod_host_tpl_init();
        assert_eq!(actions.actions().len(), 2);
        matches!(actions.actions()[0], ActionType::Gxl(_));
        matches!(actions.actions()[1], ActionType::Gxl(_));
    }

    #[test]
    fn test_k8s_tpl_init() {
        let actions = Actions::mod_k8s_tpl_init();
        assert_eq!(actions.actions().len(), 2);
        matches!(actions.actions()[0], ActionType::Gxl(_));
        matches!(actions.actions()[1], ActionType::Gxl(_));
    }

    #[test]
    fn test_save_and_load_actions() -> SpecResult<()> {
        let temp_dir = TempDir::new().owe_res()?;
        let path = temp_dir.path().to_path_buf();

        // 测试保存和加载
        let original = Actions::mod_host_tpl_init();
        original.save_to(&path)?;

        let loaded = Actions::load_from(&path)?;
        assert_eq!(loaded.actions().len(), original.actions().len());
        Ok(())
    }
}
