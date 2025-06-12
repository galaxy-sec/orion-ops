use std::path::{Path, PathBuf};

use crate::{
    workflow::prj::GxlProject,
    addr::{AddrType, GitAddr, LocalAddr},
    const_vars::VALUE_FILE,
    error::{SpecError, SpecResult},
    module::{
        CpuArch, OsCPE, RunSPC, TargetNode,
        depend::{Dependency, DependencySet},
        refs::ModuleSpecRef,
    },
    system::ModulesList,
    types::{Configable, Localizable, LocalizePath, Persistable, UpdateOptions, ValueConfable},
    vars::{ValueDict, ValueType},
};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::{Deref, DerefMut};
use log::{error, info};
use orion_error::{ErrorOwe, UvsLogicFrom};
use serde_derive::{Deserialize, Serialize};

use super::init::{MOD_APP_GAL_WORK, mod_app_gitignore};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModAppConf {
    module_list: ModulesList,
    local_envs: DependencySet,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    root_local: Option<PathBuf>,
}

#[derive(Getters, Clone, Debug)]
pub struct ModAppProject {
    conf: ModAppConf,
    project: GxlProject,
    val_dict: ValueDict,
}
impl ModAppConf {
    pub fn new(mut mod_list: ModulesList, local_res: DependencySet, root_local: PathBuf) -> Self {
        mod_list.set_mods_local(root_local.clone());
        Self {
            module_list: mod_list,
            local_envs: local_res,
            root_local: Some(root_local),
        }
    }
}
impl ModAppProject {
    pub fn new(mod_list: ModulesList, local_res: DependencySet, root_local: PathBuf) -> Self {
        let conf = ModAppConf::new(mod_list, local_res, root_local);
        let mut val_dict = ValueDict::default();
        val_dict.insert("KEY1", ValueType::from("VALUE1"));
        Self {
            conf,
            project: GxlProject::from(MOD_APP_GAL_WORK.to_string()),
            val_dict,
        }
    }
    pub fn load(root: &Path) -> SpecResult<Self> {
        let mut flag = log_guard!(
            info!(
                target : "spec/local/modprj",
                "load modprj  to {} success!", root.display()
            ),
            error!(
                target : "spec/local/modprj",
                "load modprj  to {} fail!", root.display()
            )
        );

        let conf_file = root.join("mod_loca_prj.yml");
        let mut conf = ModAppConf::from_conf(&conf_file)?;
        conf.root_local = Some(root.to_path_buf());
        let project = GxlProject::load_from(root)?;
        let val_root = root.join("value");
        let val_file = val_root.join(VALUE_FILE);
        let val_dict = ValueDict::from_valconf(&val_file)?;
        conf.module_list.set_mods_local(root.to_path_buf());
        flag.flag_suc();
        Ok(Self {
            conf,
            project,
            val_dict,
        })
    }
    pub fn save(&self, root: &Path) -> SpecResult<()> {
        let mut flag = log_guard!(
            info!(
                target : "spec/local/modprj",
                "save modprj  to {} success!", root.display()
            ),
            error!(
                target : "spec/local/modprj",
                "save modprj  to {} fail!", root.display()
            )
        );
        let conf_file = root.join("mod_loca_prj.yml");
        let val_root = root.join("value");
        let val_file = val_root.join(VALUE_FILE);
        self.conf.save_conf(&conf_file)?;
        self.project.save_to(root, None)?;
        if !val_file.exists() {
            std::fs::create_dir_all(val_root).owe_logic()?;
            self.val_dict.save_valconf(&val_file)?;
        }
        mod_app_gitignore(root)?;
        flag.flag_suc();
        Ok(())
    }

    pub fn modules(&self) -> &ModulesList {
        self.conf.module_list()
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Default)]
pub struct LocalRes {
    resource: Vec<Dependency>,
}

impl ModAppConf {
    pub async fn update(&self) -> SpecResult<()> {
        if let Some(path) = &self.root_local {
            let options = &UpdateOptions::default();
            self.module_list.update(path, options).await?;
            self.local_envs.update().await?;
            Ok(())
        } else {
            Err(SpecError::from_logic("local paths not setting ".into()))
        }
    }
}

impl ModAppProject {
    pub async fn update(&self) -> SpecResult<()> {
        self.conf.update().await
    }
}

#[async_trait]
impl Localizable for ModAppConf {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        if let Some(path) = &self.root_local {
            let local_path = LocalizePath::from_root(path);
            self.module_list().localize(Some(local_path)).await?;
            Ok(())
        } else {
            Err(SpecError::from_logic("local paths not setting ".into()))
        }
    }
}

#[async_trait]
impl Localizable for ModAppProject {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        self.conf.localize(_dst_path).await?;
        Ok(())
    }
}

pub fn make_mod_cust_testins(prj_path: &Path) -> SpecResult<ModAppProject> {
    let mod_name = "postgresql";
    let mut mod_list = ModulesList::default();
    mod_list.add_ref(ModuleSpecRef::from(
        mod_name,
        LocalAddr::from("./example/modules/postgresql"),
        TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
    ));

    let mut res = DependencySet::default();
    res.push(
        Dependency::new(
            AddrType::from(GitAddr::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            prj_path.join("env_res"),
        )
        .with_rename("bit-common"),
    );
    Ok(ModAppProject::new(mod_list, res, prj_path.to_path_buf()))
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        const_vars::MODULES_INS_ROOT,
        error::SpecResult,
        app_mod::app::{ModAppProject, make_mod_cust_testins},
        tools::test_init,
        types::Localizable,
    };

    #[tokio::test]
    async fn test_mod_cust_prj_running() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(MODULES_INS_ROOT).join("mod_cust-prj");
        let project = make_mod_cust_testins(&prj_path).assert("make cust");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("ok");
        }
        std::fs::create_dir_all(&prj_path).assert("yes");
        project.save(&prj_path).assert("save dss_prj");
        let project = ModAppProject::load(&prj_path).assert("dss-project");
        project.update().await.assert("spec.update_local");
        project.localize(None).await.assert("spec.localize");
        Ok(())
    }
}
