use std::path::Path;

use orion_error::{ErrorOwe, ErrorWith};

use crate::{
    error::SpecResult,
    task::OperationType,
    workflow::{
        act::{Workflow, Workflows},
        gxl::GxlAction,
        prj::GxlProject,
    },
};

const SYS_SETUP_GXL: &str = include_str!("init/workflows/setup.gxl");
const SYS_UPDATE_GXL: &str = include_str!("init/workflows/update.gxl");
const SYS_SPC_PRJ: &str = include_str!("init/_gal/work.gxl");
const SYS_GITIGNORE: &str = include_str!("init/.gitignore");

pub trait SysActIniter {
    fn sys_setup_tpl() -> Self;
    fn sys_update_tpl() -> Self;
}
pub trait SysPrjIniter {
    fn spec_tpl() -> Self;
}

impl SysActIniter for GxlAction {
    fn sys_setup_tpl() -> Self {
        Self::new(
            OperationType::Setup,
            "setup.gxl".into(),
            SYS_SETUP_GXL.to_string(),
        )
    }
    fn sys_update_tpl() -> Self {
        Self::new(
            OperationType::Update,
            "update.gxl".into(),
            SYS_UPDATE_GXL.to_string(),
        )
    }
}
impl SysPrjIniter for GxlProject {
    fn spec_tpl() -> Self {
        Self::from(SYS_SPC_PRJ.to_string())
    }
}

pub trait SysIniter {
    fn sys_tpl_init() -> Self;
}

impl<T> SysIniter for Workflows<T> {
    fn sys_tpl_init() -> Self {
        let project = GxlProject::spec_tpl();
        let actions = vec![
            Workflow::Gxl(GxlAction::sys_setup_tpl()),
            Workflow::Gxl(GxlAction::sys_update_tpl()),
        ];
        Self::new(project, actions)
    }
}

pub fn sys_init_gitignore(path: &Path) -> SpecResult<()> {
    let ignore_path = path.join(".gitignore");
    if !ignore_path.exists() {
        std::fs::write(&ignore_path, SYS_GITIGNORE)
            .owe_res()
            .with(&ignore_path)?;
    }
    Ok(())
}
