use crate::{
    action::{
        act::{ModWorkflows, Workflow},
        bsh::BashAction,
        gxl::GxlAction,
        prj::GxlProject,
    },
    task::OperationType,
};

const HOST_SETUP_GXL: &str = include_str!("init/host/workflows/setup.gxl");
const HOST_UPDATE_GXL: &str = include_str!("init/host/workflows/update.gxl");
const HOST_SETUP_SH: &str = include_str!("init/host/workflows/setup.sh");
const MOD_HOST_PRJ: &str = include_str!("init/host/_gal/work.gxl");

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
        Self::new(OperationType::Setup, HOST_SETUP_GXL.to_string())
    }
    fn host_update_tpl() -> Self {
        Self::new(OperationType::Update, HOST_UPDATE_GXL.to_string())
    }
    fn k8s_setup_tpl() -> Self {
        Self::new(OperationType::Setup, K8S_SETUP_GXL.to_string())
    }
    fn k8s_update_tpl() -> Self {
        Self::new(OperationType::Update, K8S_UPDATE_GXL.to_string())
    }
}
impl ModPrjIniter for GxlProject {
    fn spec_host_tpl() -> Self {
        Self::from(MOD_HOST_PRJ.to_string())
    }
    fn spec_k8s_tpl() -> Self {
        Self::from(MOD_K8S_PRJ.to_string())
    }
}

impl BashAction {
    pub fn setup_tpl() -> Self {
        Self::new(OperationType::Setup, HOST_SETUP_SH.to_string())
    }
}

pub trait ModIniter {
    fn mod_host_tpl_init() -> Self;
    fn mod_k8s_tpl_init() -> Self;
}

impl ModIniter for ModWorkflows {
    fn mod_host_tpl_init() -> Self {
        let project = GxlProject::spec_host_tpl();
        let actions = vec![
            Workflow::Gxl(GxlAction::host_setup_tpl()),
            Workflow::Gxl(GxlAction::host_update_tpl()),
        ];
        Self::new(project, actions)
    }

    fn mod_k8s_tpl_init() -> ModWorkflows {
        let project = GxlProject::spec_k8s_tpl();
        let actions = vec![
            Workflow::Gxl(GxlAction::k8s_setup_tpl()),
            Workflow::Gxl(GxlAction::k8s_update_tpl()),
        ];
        Self::new(project, actions)
    }
}
