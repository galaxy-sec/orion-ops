use std::path::PathBuf;

use crate::{
    action::{act::SysWorkflows, prj::GxlProject},
    addr::{AddrType, GitAddr, LocalAddr},
    error::SpecResult,
    module::{
        CpuArch, OsCPE, RunSPC, TargetNode,
        refs::{DependItem, ModuleSpecRef},
    },
    system::ModulesList,
    types::{Configable, Localizable, LocalizePath, Persistable, UpdateOptions},
};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::{Deref, DerefMut};
use serde_derive::{Deserialize, Serialize};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModLocallyConf {
    mod_list: ModulesList,
    local_res: LocalRes,
    root_local: PathBuf,
}

#[derive(Getters, Clone, Debug)]
pub struct ModLocallyProject {
    conf: ModLocallyConf,
    project: GxlProject,
}
impl ModLocallyConf {
    pub fn new(mut mod_list: ModulesList, local_res: LocalRes, root_local: PathBuf) -> Self {
        mod_list.set_mods_local(root_local.clone());
        Self {
            mod_list,
            local_res,
            root_local,
        }
    }
}
impl ModLocallyProject {
    pub fn new(mut mod_list: ModulesList, local_res: LocalRes, root_local: PathBuf) -> Self {
        let conf = ModLocallyConf::new(mod_list, local_res, root_local);
        Self {
            conf,
            project: GxlProject::from("".to_string()),
        }
    }
    pub fn load(path: &PathBuf) -> SpecResult<Self> {
        let mut conf = ModLocallyConf::from_conf(&path)?;
        let project = GxlProject::load_from(path)?;
        conf.mod_list.set_mods_local(conf.root_local.clone());
        Ok(Self { conf, project })
    }
    pub fn save(&self, root: PathBuf) -> SpecResult<()> {
        let conf_file = root.join("mod_locally_prj.yml");
        self.save_conf(&conf_file)?;
        Ok(())
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Default)]
pub struct LocalRes {
    resource: Vec<DependItem>,
}

impl ModLocallyProject {
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
impl Localizable for ModLocallyProject {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        let local_path = LocalizePath::from_root(self.root_local());
        self.mod_list().localize(Some(local_path)).await?;
        Ok(())
    }
}

pub fn make_mod_cust_example(prj_path: PathBuf) -> SpecResult<ModLocallyProject> {
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
    Ok(ModLocallyProject::new(mod_list, res, prj_path.clone()))
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        const_vars::MODULES_INS_ROOT,
        cust::{
            modprj::{ModLocallyProject, make_mod_cust_example},
            sysproj::SysCustProject,
        },
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
        let project = ModLocallyProject::load(&conf_file).assert("dss-project");
        project.update().await.assert("spec.update_local");
        project.localize(None).await.assert("spec.localize");
        Ok(())
    }
}
