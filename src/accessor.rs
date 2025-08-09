use log::error;
use orion_common::serde::Yamlable;
use orion_variate::{
    addr::{
        access_ctrl::serv::NetAccessCtrl,
        accessor::{UniversalAccessor, UniversalConfig},
    },
    vars::{EnvDict, EnvEvalable},
};
use std::{env::home_dir, sync::Arc};

use crate::const_vars::NET_ACCS_CTRL_FILE;

pub fn build_accessor(dict: &EnvDict) -> UniversalAccessor {
    if let Some(path) = home_dir().map(|x| x.join(NET_ACCS_CTRL_FILE))
        && path.exists()
    {
        match NetAccessCtrl::from_yml(&path) {
            Ok(redirect) => {
                let ctrl = redirect.env_eval(dict);
                return UniversalAccessor::new(UniversalConfig::default().with_ctrl(ctrl));
            }
            Err(e) => {
                error!("load redirect conf failed!\npath:{} \n{e}", path.display());
            }
        }
    }
    UniversalAccessor::new(UniversalConfig::default())
}
pub fn accessor_for_test() -> Arc<UniversalAccessor> {
    Arc::new(build_accessor(&EnvDict::default()))
}

pub fn accessor_for_default() -> Arc<UniversalAccessor> {
    Arc::new(build_accessor(&EnvDict::default()))
}
