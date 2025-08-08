use crate::const_vars::OPS_PRJ_CONF_FILE;
use crate::error::OpsReason;
use crate::predule::*;
use crate::system::refs::SysModelSpecRef;
use crate::{error::MainResult, module::depend::DependencySet};
use orion_common::serde::Configable;

use crate::types::{Accessor, InsUpdateable, RefUpdateable};
use async_trait::async_trait;
use orion_infra::auto_exit_log;
use orion_variate::addr::LocalPath;
use orion_variate::update::DownloadOptions;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConf {
    name: String,
    work_envs: DependencySet,
}

impl ProjectConf {
    pub fn new<S: Into<String>>(name: S, local_res: DependencySet) -> Self {
        Self {
            name: name.into(),
            work_envs: local_res,
        }
    }
    pub fn for_test() -> Self {
        let _systems = vec![SysModelSpecRef::from(
            "example_sys",
            LocalPath::from("./example/sys-model-spec/example_sys"),
        )];
        let work_envs = DependencySet::example();
        Self {
            name: "example_sys".to_string(),
            work_envs,
        }
    }
    pub fn load(path: &Path) -> MainResult<Self> {
        let conf_file = path.join(OPS_PRJ_CONF_FILE);
        let ins = Self::from_conf(&conf_file).owe_conf()?;
        Ok(ins)
    }
}
#[async_trait]
impl InsUpdateable<ProjectConf> for ProjectConf {
    async fn update_local(
        mut self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<Self> {
        let mut flag = auto_exit_log!(
            info!(
                target : "ops-prj/conf",
                "ins conf update from {} success!", path.display()
            ),
            error!(
                target : "ops-prj/conf",
                "ins conf update from {} fail!", path.display()
            )
        );
        self.work_envs
            .update_local(accessor, path, options)
            .await
            .owe(OpsReason::Update.into())?;
        flag.mark_suc();
        Ok(self)
    }
}
