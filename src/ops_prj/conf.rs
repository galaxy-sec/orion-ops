use crate::error::OpsReason;
use crate::predule::*;

use crate::system::refs::SysModelSpecRef;
use crate::{
    error::SpecResult,
    module::depend::DependencySet,
    types::{Configable, Localizable},
};
const OPS_PRJ_FILE: &str = "ops-prj.yml";

use crate::types::{LocalizeOptions, SysUpdateable};
use async_trait::async_trait;
use orion_infra::auto_exit_log;
use orion_x::addr::LocalAddr;
use orion_x::types::ValuePath;
use orion_x::update::UpdateOptions;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConf {
    name: String,
    systems: Vec<SysModelSpecRef>,
    work_envs: DependencySet,
}

impl ProjectConf {
    pub fn new<S: Into<String>>(name: S, local_res: DependencySet) -> Self {
        Self {
            name: name.into(),
            work_envs: local_res,
            systems: Vec::new(),
        }
    }
    pub fn for_test() -> Self {
        let systems = vec![SysModelSpecRef::from(
            "example_sys",
            LocalAddr::from("./example/sys-model-spec/example_sys"),
        )];
        let work_envs = DependencySet::example();
        Self {
            name: "example_sys".to_string(),
            systems,
            work_envs,
        }
    }
    pub fn load(path: &Path) -> SpecResult<Self> {
        let conf_file = path.join(OPS_PRJ_FILE);
        let mut ins = Self::from_conf(&conf_file)?;
        let mut updated_sys = Vec::new();
        for mut sys in ins.systems {
            if sys.is_update(path) {
                sys = sys.load(path)?;
            }
            updated_sys.push(sys);
        }
        ins.systems = updated_sys;
        Ok(ins)
    }
}
#[async_trait]
impl SysUpdateable<ProjectConf> for ProjectConf {
    async fn update_local(mut self, path: &Path, options: &UpdateOptions) -> SpecResult<Self> {
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
            .update(options)
            .await
            .owe(OpsReason::Update.into())?;
        let mut updated_sys = Vec::new();
        for sys in self.systems {
            updated_sys.push(sys.update_local(path, options).await?);
        }
        self.systems = updated_sys;
        flag.mark_suc();
        Ok(self)
    }
}
#[async_trait]
impl Localizable for ProjectConf {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        for sys in self.systems() {
            sys.localize(dst_path.clone(), options.clone()).await?;
        }
        Ok(())
    }
}
