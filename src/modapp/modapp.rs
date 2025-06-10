use std::path::PathBuf;

use crate::{
    action::prj::GxlProject,
    addr::{AddrType, GitAddr, LocalAddr},
    const_vars::VALUE_FILE,
    error::SpecResult,
    module::{
        CpuArch, OsCPE, RunSPC, TargetNode,
        refs::{DependItem, ModuleSpecRef},
    },
    system::ModulesList,
    types::{Configable, Localizable, LocalizePath, Persistable, UpdateOptions, ValueConfable},
    vars::{ValueDict, ValueType},
};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::{Deref, DerefMut};
use log::{error, info};
use orion_error::ErrorOwe;
use serde_derive::{Deserialize, Serialize};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModAppConf {
    mod_list: ModulesList,
    local_res: LocalRes,
    root_local: PathBuf,
}

#[derive(Getters, Clone, Debug)]
pub struct ModAppProject {
    conf: ModAppConf,
    project: GxlProject,
    val_dict: ValueDict,
}
impl ModAppConf {
    pub fn new(mut mod_list: ModulesList, local_res: LocalRes, root_local: PathBuf) -> Self {
        mod_list.set_mods_local(root_local.clone());
        Self {
            mod_list,
            local_res,
            root_local,
        }
    }
}
impl ModAppProject {
    pub fn new(mod_list: ModulesList, local_res: LocalRes, root_local: PathBuf) -> Self {
        let conf = ModAppConf::new(mod_list, local_res, root_local);
        let mut val_dict = ValueDict::default();
        val_dict.insert("KEY1", ValueType::from("VALUE1"));
        Self {
            conf,
            project: GxlProject::from("".to_string()),
            val_dict,
        }
    }
    pub fn load(root: &PathBuf) -> SpecResult<Self> {
        let mut flag = log_flag!(
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
        let project = GxlProject::load_from(root)?;
        let val_root = root.join("value");
        let val_file = val_root.join(VALUE_FILE);
        let val_dict = ValueDict::from_valconf(&val_file)?;
        conf.mod_list.set_mods_local(conf.root_local.clone());
        flag.flag_suc();
        Ok(Self {
            conf,
            project,
            val_dict,
        })
    }
    pub fn save(&self, root: &PathBuf) -> SpecResult<()> {
        let mut flag = log_flag!(
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
        flag.flag_suc();
        if !val_file.exists() {
            std::fs::create_dir_all(val_root).owe_logic()?;
            self.val_dict.save_valconf(&val_file)?;
        }
        Ok(())
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Default)]
pub struct LocalRes {
    resource: Vec<DependItem>,
}

impl ModAppConf {
    pub async fn update(&self) -> SpecResult<()> {
        let path = &self.root_local;
        let options = &UpdateOptions::default();
        self.mod_list.update(path, options).await?;
        for res in self.local_res.iter() {
            if res.is_enable() {
                res.update(options).await?;
            }
        }
        Ok(())
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
        let local_path = LocalizePath::from_root(self.root_local());
        self.mod_list().localize(Some(local_path)).await?;
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModAppProject {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        self.conf.localize(_dst_path).await?;
        Ok(())
    }
}

pub fn make_mod_cust_testins(prj_path: &PathBuf) -> SpecResult<ModAppProject> {
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
    Ok(ModAppProject::new(mod_list, res, prj_path.clone()))
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        const_vars::MODULES_INS_ROOT,
        error::SpecResult,
        modapp::modapp::{ModAppProject, make_mod_cust_testins},
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
