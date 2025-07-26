use crate::const_vars::SYS_MODLE_DEF_YML;
use std::path::PathBuf;

use crate::const_vars::{MOD_LIST_YML, VARS_YML};
use getset::Getters;

#[derive(Getters, Clone, Debug)]
#[getset(get = "pub ")]
pub struct SysTargetPaths {
    target_root: PathBuf,
    define_path: PathBuf,
    spec_path: PathBuf,
    //net_path: PathBuf,
    //res_path: PathBuf,
    vars_path: PathBuf,
    modlist_path: PathBuf,
    workflow_path: PathBuf,
}
impl From<&PathBuf> for SysTargetPaths {
    fn from(target_root: &PathBuf) -> Self {
        //let spec_path = target_root.join(SPEC_DIR);
        Self {
            target_root: target_root.to_path_buf(),
            define_path: target_root.join(SYS_MODLE_DEF_YML),
            //net_path: target_root.join(NET_RES_YML),
            //res_path: target_root.join(RESOURCE_YML),
            vars_path: target_root.join(VARS_YML),
            modlist_path: target_root.join(MOD_LIST_YML),
            workflow_path: target_root.to_path_buf(),
            spec_path: target_root.clone(),
        }
    }
}
