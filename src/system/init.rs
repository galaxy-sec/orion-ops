use crate::{
    action::{
        act::{ModWorkflows, Workflow, Workflows},
        gxl::GxlAction,
        prj::GxlProject,
    },
    task::OperationType,
};

const SYS_SETUP_GXL: &str = include_str!("init/actions/setup.gxl");
const SYS_UPDATE_GXL: &str = include_str!("init/actions/update.gxl");
const SYS_SPC_PRJ: &str = include_str!("init/_gal/work.gxl");

pub trait SysActIniter {
    fn sys_setup_tpl() -> Self;
    fn sys_update_tpl() -> Self;
}
pub trait SysPrjIniter {
    fn spec_tpl() -> Self;
}

impl SysActIniter for GxlAction {
    fn sys_setup_tpl() -> Self {
        Self::new(OperationType::Setup, SYS_SETUP_GXL.to_string())
    }
    fn sys_update_tpl() -> Self {
        Self::new(OperationType::Update, SYS_UPDATE_GXL.to_string())
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
