use std::path::Path;

use orion_error::{ErrorOwe, ErrorWith};

use crate::{
    error::SpecResult,
    task::OperationType,
    workflow::{
        act::{ModWorkflows, Workflow},
        gxl::GxlAction,
        prj::GxlProject,
    },
};

const HOST_SETUP_GXL: &str = include_str!("init/host/workflows/setup.gxl");
const HOST_UPDATE_GXL: &str = include_str!("init/host/workflows/update.gxl");
pub const MOD_PRJ_WORK_GXL: &str = include_str!("init/_gal/work.gxl");
pub const MOD_PRJ_ADM_GXL: &str = include_str!("init/_gal/adm.gxl");
const MOD_HOST_PRJ: &str = include_str!("init/host/_gal/work.gxl");
const MOD_GITIGNORE: &str = include_str!("init/.gitignore");

const K8S_SETUP_GXL: &str = include_str!("init/k8s/spec/workflows/setup.gxl");
const K8S_UPDATE_GXL: &str = include_str!("init/k8s/spec/workflows/update.gxl");
const MOD_K8S_PRJ: &str = include_str!("init/k8s/_gal/work.gxl");
pub trait ModActIniter {
    fn host_setup_tpl() -> Self;
    fn host_update_tpl() -> Self;
    fn k8s_setup_tpl() -> Self;
    fn k8s_update_tpl() -> Self;
}
pub trait ModPrjIniter {
    fn spec_host_tpl() -> Self;
    fn spec_k8s_tpl() -> Self;
}

impl ModActIniter for GxlAction {
    fn host_setup_tpl() -> Self {
        Self::new(
            OperationType::Setup,
            "setup.gxl".into(),
            HOST_SETUP_GXL.to_string(),
        )
    }
    fn host_update_tpl() -> Self {
        Self::new(
            OperationType::Update,
            "update.gxl".into(),
            HOST_UPDATE_GXL.to_string(),
        )
    }
    fn k8s_setup_tpl() -> Self {
        Self::new(
            OperationType::Setup,
            "setup.gxl".into(),
            K8S_SETUP_GXL.to_string(),
        )
    }
    fn k8s_update_tpl() -> Self {
        Self::new(
            OperationType::Update,
            "update.gxl".into(),
            K8S_UPDATE_GXL.to_string(),
        )
    }
}
impl ModPrjIniter for GxlProject {
    fn spec_host_tpl() -> Self {
        Self::from(MOD_HOST_PRJ)
    }
    fn spec_k8s_tpl() -> Self {
        Self::from(MOD_K8S_PRJ)
    }
}

pub trait ModIniter {
    fn mod_host_tpl_init() -> Self;
    fn mod_k8s_tpl_init() -> Self;
}

impl ModIniter for ModWorkflows {
    fn mod_host_tpl_init() -> Self {
        let actions = vec![
            Workflow::Gxl(GxlAction::host_setup_tpl()),
            Workflow::Gxl(GxlAction::host_update_tpl()),
        ];
        Self::new(actions)
    }

    fn mod_k8s_tpl_init() -> ModWorkflows {
        let actions = vec![
            Workflow::Gxl(GxlAction::k8s_setup_tpl()),
            Workflow::Gxl(GxlAction::k8s_update_tpl()),
        ];
        Self::new(actions)
    }
}

pub fn mod_init_gitignore(path: &Path) -> SpecResult<()> {
    let ignore_path = path.join(".gitignore");
    if !ignore_path.exists() {
        std::fs::write(&ignore_path, MOD_GITIGNORE)
            .owe_res()
            .with(&ignore_path)?;
    }
    Ok(())
}
