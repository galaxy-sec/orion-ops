use std::path::PathBuf;

use crate::{
    addr::{AddrType, GitAddr},
    error::SpecResult,
    module::depend::{DependItem, DependVec},
    system::{refs::SysModelSpecRef, spec::SysModelSpec},
    types::{AsyncUpdateable, Localizable, LocalizePath, UpdateOptions},
};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::{Deref, DerefMut};
use serde_derive::{Deserialize, Serialize};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysCustProject {
    model_spec: SysModelSpecRef,
    local_res: DependVec,
    root_local: PathBuf,
}
impl SysCustProject {
    pub fn new(model_spec: SysModelSpecRef, local_res: DependVec, root_local: PathBuf) -> Self {
        Self {
            model_spec,
            local_res,
            root_local,
        }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Default)]
pub struct LocalRes {
    resource: Vec<DependItem>,
}

impl SysCustProject {
    pub async fn update(&self) -> SpecResult<()> {
        let path = &self.root_local;
        let options = &UpdateOptions::default();
        self.model_spec
            .update_rename(path, "system", options)
            .await?;
        self.local_res.update().await?;
        Ok(())
    }
}

#[async_trait]
impl Localizable for SysCustProject {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        let options = &UpdateOptions::default();
        let sys_path = self.root_local().join("system");
        let spec = SysModelSpec::load_from(&sys_path)?;
        spec.update_local(options).await?;
        let local_path = LocalizePath::from_root(self.root_local());
        spec.localize(Some(local_path)).await?;
        Ok(())
    }
}

pub fn make_sys_cust_example(prj_path: PathBuf) -> SpecResult<SysCustProject> {
    let target = "example-sys-x1";
    let spec_ref = SysModelSpecRef::from(
        target,
        GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/spec_example_sys.git")
            .path("example-sys-x1"),
    );
    let mut res = DependVec::default();
    res.push(
        DependItem::new(
            AddrType::from(GitAddr::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            prj_path.join("env_res"),
        )
        .with_rename("bit-common"),
    );
    Ok(SysCustProject::new(spec_ref, res, prj_path.clone()))
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        addr::{AddrType, LocalAddr},
        const_vars::{SYS_MODEL_INS_ROOT, SYS_MODEL_SPC_ROOT},
        error::SpecResult,
        m_app::sysproj::SysCustProject,
        module::depend::{DependItem, DependVec},
        system::refs::SysModelSpecRef,
        tools::test_init,
        types::{Configable, Localizable},
    };

    #[tokio::test]
    async fn test_cust_prj_running() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(SYS_MODEL_INS_ROOT).join("dss-prj-1");
        let target = "example-sys";
        let spec_ref = SysModelSpecRef::from(
            target,
            LocalAddr::from(format!("{}/{}", SYS_MODEL_SPC_ROOT, "example-sys")),
        );

        let mut res = DependVec::default();
        res.push(
            DependItem::new(
                AddrType::from(LocalAddr::from("./example/knowlege/mysql")),
                prj_path.join("env_res"),
            )
            .with_rename("mysql2"),
        );
        let project = SysCustProject::new(spec_ref, res, prj_path.clone());

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
