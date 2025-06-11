use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use super::gxl::GxlAction;
use super::prj::GxlProject;
use derive_getters::Getters;
use log::warn;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom};

use crate::{error::SpecResult, types::Persistable};

#[derive(Getters, Clone, Debug)]
pub struct Workflows<T> {
    project: GxlProject,
    actions: Vec<Workflow>,
    _phantom: PhantomData<T>,
}

pub trait FlowPaths {
    fn workflow() -> PathBuf;
}
#[derive(Clone, Debug)]
pub struct ModLabel;
impl FlowPaths for ModLabel {
    fn workflow() -> PathBuf {
        PathBuf::from(crate::const_vars::WORKFLOWS_DIR)
    }
}
#[derive(Clone, Debug)]
pub struct SysLabel;

impl FlowPaths for SysLabel {
    fn workflow() -> PathBuf {
        PathBuf::from(crate::const_vars::WORKFLOWS_DIR)
    }
}
pub type ModWorkflows = Workflows<ModLabel>;
pub type SysWorkflows = Workflows<SysLabel>;

impl<T> Workflows<T> {
    pub fn new(project: GxlProject, actions: Vec<Workflow>) -> Self {
        Self {
            project,
            actions,
            _phantom: PhantomData,
        }
    }
}

impl<T> Persistable<Workflows<T>> for Workflows<T>
where
    T: FlowPaths,
{
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()> {
        let action_path = path.join(T::workflow());
        std::fs::create_dir_all(&action_path)
            .owe_res()
            .with(&action_path)?;
        for item in &self.actions {
            item.save_to(&action_path, name.clone())?;
        }
        self.project.save_to(path, name)?;
        Ok(())
    }

    //加载 path 目录的文件
    fn load_from(path: &Path) -> SpecResult<Self> {
        let mut actions = Vec::new();
        let actions_path = path.join(T::workflow());
        for entry in std::fs::read_dir(&actions_path)
            .owe_res()
            .with(&actions_path)
            .want("read workflows")
            .with(("workflow", "read workflows"))?
        {
            let entry = entry.owe_res()?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                let action = Workflow::load_from(&entry_path);
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
        Ok(Workflows {
            project,
            actions,
            _phantom: PhantomData,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Workflow {
    Gxl(GxlAction),
}

impl Persistable<Workflow> for Workflow {
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()> {
        match self {
            Workflow::Gxl(act) => act.save_to(path, name),
        }
    }

    fn load_from(path: &Path) -> SpecResult<Workflow> {
        // 首先检查文件是否存在且是普通文件
        if !path.exists() {
            return Err(StructError::from_conf("path not exists".into())).with(path);
        }

        if !path.is_file() {
            return Err(StructError::from_conf("path not file".into())).with(path);
        }

        // 根据扩展名分发加载逻辑
        match path.extension().and_then(|s| s.to_str()) {
            Some("gxl") => GxlAction::load_from(path).map(Workflow::Gxl),
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
        let actions = ModWorkflows::mod_host_tpl_init();
        assert_eq!(actions.actions().len(), 2);
        matches!(actions.actions()[0], Workflow::Gxl(_));
        matches!(actions.actions()[1], Workflow::Gxl(_));
    }

    #[test]
    fn test_k8s_tpl_init() {
        let actions = ModWorkflows::mod_k8s_tpl_init();
        assert_eq!(actions.actions().len(), 2);
        matches!(actions.actions()[0], Workflow::Gxl(_));
        matches!(actions.actions()[1], Workflow::Gxl(_));
    }

    #[test]
    fn test_save_and_load_actions() -> SpecResult<()> {
        let temp_dir = TempDir::new().owe_res()?;
        let path = temp_dir.path().to_path_buf();

        // 测试保存和加载
        let original = ModWorkflows::mod_host_tpl_init();
        original.save_to(&path, None)?;

        let loaded = ModWorkflows::load_from(&path)?;
        assert_eq!(loaded.actions().len(), original.actions().len());
        Ok(())
    }
}
