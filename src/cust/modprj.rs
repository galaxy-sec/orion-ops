use std::path::PathBuf;

use crate::{
    addr::{AddrType, GitAddr, LocalAddr},
    error::SpecResult,
    module::{
        CpuArch, OsCPE, RunSPC, TargetNode,
        refs::{DependItem, ModuleSpecRef},
    },
    system::{ModulesList, refs::SysModelSpecRef, spec::SysModelSpec},
    types::{AsyncUpdateable, Localizable, LocalizePath, UpdateOptions},
};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::{Deref, DerefMut};
use serde_derive::{Deserialize, Serialize};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModCustProject {
    mod_list: ModulesList,
    local_res: LocalRes,
    root_local: PathBuf,
}
impl ModCustProject {
    pub fn new(model_spec: ModulesList, local_res: LocalRes, root_local: PathBuf) -> Self {
        Self {
            mod_list: model_spec,
            local_res,
            root_local,
        }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Default)]
pub struct LocalRes {
    resource: Vec<DependItem>,
}

impl ModCustProject {
    pub async fn update(&self) -> SpecResult<()> {
        let path = &self.root_local;
        let options = &UpdateOptions::default();
        self.mod_list.update(path, options).await?;
        for res in self.local_res.iter() {
            res.update(options).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModCustProject {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        let local_path = LocalizePath::from_root(self.root_local());
        self.mod_list().localize(Some(local_path)).await?;
        Ok(())
    }
}

pub fn make_mod_cust_example(prj_path: PathBuf) -> SpecResult<ModCustProject> {
    let mod_name = "postgresql";
    let mut mod_list = ModulesList::default();
    mod_list.add_ref(ModuleSpecRef::from(
        mod_name,
        LocalAddr::from("./example/modules/postgresql"),
        TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
    ));

    let mut res = LocalRes::default();
    res.push(
        DependItem::new(
            AddrType::from(GitAddr::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            prj_path.join("env_res"),
        )
        .with_rename("bit-common"),
    );
    Ok(ModCustProject::new(mod_list, res, prj_path.clone()))
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        const_vars::MODULES_INS_ROOT,
        cust::{modprj::make_mod_cust_example, sysproj::SysCustProject},
        error::SpecResult,
        tools::test_init,
        types::{Configable, Localizable},
    };

    #[tokio::test]
    async fn test_mod_cust_prj_running() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(MODULES_INS_ROOT).join("mod_cust-prj");
        let project = make_mod_cust_example(prj_path.clone()).assert("make cust");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("ok");
        }
        std::fs::create_dir_all(&prj_path).assert("yes");
        let conf_file = prj_path.join("sys_cust_prj.yml");
        project.save_conf(&conf_file).assert("save dss_prj");
        let project = SysCustProject::from_conf(&conf_file).assert("dss-project");
        project.update().await.assert("spec.update_local");
        project.localize(None).await.assert("spec.localize");
        Ok(())
    }
}
