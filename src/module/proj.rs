use crate::predule::*;

use super::init::{MOD_PRJ_ADM_GXL, MOD_PRJ_WORK_GXL, mod_init_gitignore};
use crate::types::LocalizeOptions;
use crate::{
    addr::{AddrType, GitAddr, types::EnvVarPath},
    const_vars::MODULES_SPC_ROOT,
    module::{
        depend::{Dependency, DependencySet},
        spec::ModuleSpec,
    },
    tools::make_clean_path,
    types::{AsyncUpdateable, Configable, Localizable, LocalizePath, Persistable},
    vars::{ValueDict, ValueType},
    workflow::prj::GxlProject,
};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModConf {
    test_envs: DependencySet,
}

#[derive(Getters, Clone, Debug)]
pub struct ModProject {
    conf: ModConf,
    mod_spec: ModuleSpec,
    project: GxlProject,
    root_local: PathBuf,
}
impl ModConf {
    pub fn new(local_res: DependencySet) -> Self {
        Self {
            test_envs: local_res,
        }
    }
}
impl ModProject {
    pub fn new(spec: ModuleSpec, local_res: DependencySet, root_local: PathBuf) -> Self {
        let conf = ModConf::new(local_res);
        let mut val_dict = ValueDict::default();
        val_dict.insert("KEY1", ValueType::from("VALUE1"));
        Self {
            conf,
            mod_spec: spec,
            project: GxlProject::from((MOD_PRJ_WORK_GXL, MOD_PRJ_ADM_GXL)),
            root_local,
        }
    }
    pub fn load(root_local: &Path) -> SpecResult<Self> {
        let mut flag = log_guard!(
            info!(
                target : "/mod_prj",
                "load modprj  to {} success!", root_local.display()
            ),
            error!(
                target : "/mod_prj",
                "load modprj  to {} fail!", root_local.display()
            )
        );

        let conf_file = root_local.join("mod_prj.yml");
        let conf = ModConf::from_conf(&conf_file)?;
        let root_local = root_local.to_path_buf();
        let mod_spec = ModuleSpec::load_from(&root_local)?;
        let project = GxlProject::load_from(&root_local)?;
        flag.flag_suc();
        Ok(Self {
            conf,
            mod_spec,
            project,
            root_local,
        })
    }
    pub fn save(&self) -> SpecResult<()> {
        let mut flag = log_guard!(
            info!(
                target : "spec/local/modprj",
                "save modprj  to {} success!", self.root_local().display()
            ),
            error!(
                target : "spec/local/modprj",
                "save modprj  to {} fail!", self.root_local().display()
            )
        );
        let conf_file = self.root_local().join("mod_prj.yml");
        self.conf.save_conf(&conf_file)?;
        self.mod_spec
            .save_to(self.root_local(), Some("./".into()))?;
        self.project.save_to(self.root_local(), None)?;
        mod_init_gitignore(self.root_local())?;
        flag.flag_suc();
        Ok(())
    }
}

impl ModConf {
    pub async fn update(&self, options: &UpdateOptions) -> SpecResult<()> {
        self.test_envs.update(options).await
    }
}

impl ModProject {
    pub async fn update(&self, options: &UpdateOptions) -> SpecResult<()> {
        self.conf.update(options).await?;
        self.mod_spec()
            .update_local(self.root_local(), options)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModConf {
    async fn localize(
        &self,
        _dst_path: Option<LocalizePath>,
        _options: LocalizeOptions,
    ) -> SpecResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModProject {
    async fn localize(
        &self,
        dst_path: Option<LocalizePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        //let local_path = LocalizePath::from_root(self.root_local());
        self.conf
            .localize(dst_path.clone(), options.clone())
            .await?;
        self.mod_spec().localize(dst_path, options).await?;
        Ok(())
    }
}
impl ModProject {
    pub fn make_new(prj_path: &Path, name: &str) -> SpecResult<Self> {
        let mod_spec = ModuleSpec::make_new(name)?;
        let res = DependencySet::default();
        Ok(ModProject::new(mod_spec, res, prj_path.to_path_buf()))
    }
    pub fn make_test_prj(name: &str) -> SpecResult<Self> {
        let prj_path = PathBuf::from(MODULES_SPC_ROOT).join(name);
        make_clean_path(&prj_path)?;
        let proj = ModProject::make_new(&prj_path, name)?;
        proj.save()?;
        Ok(proj)
    }
}

pub fn make_mod_prj_testins(prj_path: &Path) -> SpecResult<ModProject> {
    let mod_spec = ModuleSpec::for_example();
    let mut res = DependencySet::default();
    res.push(
        Dependency::new(
            AddrType::from(GitAddr::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            EnvVarPath::from(prj_path.join("test_res")),
        )
        .with_rename("bit-common"),
    );
    Ok(ModProject::new(mod_spec, res, prj_path.to_path_buf()))
}

#[cfg(test)]
pub mod tests {
    use crate::{predule::*, types::LocalizeOptions};
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        const_vars::MODULES_SPC_ROOT,
        module::proj::{ModProject, make_mod_prj_testins},
        tools::{make_clean_path, test_init},
        types::Localizable,
    };
    #[tokio::test]
    async fn test_mod_prj_new() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(MODULES_SPC_ROOT).join("mod-new");
        make_clean_path(&prj_path)?;
        let proj = ModProject::make_new(&prj_path, "mod_new")?;
        proj.save()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mod_prj_example() -> SpecResult<()> {
        test_init();

        let prj_path = PathBuf::from(MODULES_SPC_ROOT).join("postgresql");
        let project = make_mod_prj_testins(&prj_path).assert("make cust");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("ok");
        }
        std::fs::create_dir_all(&prj_path).assert("yes");
        project.save().assert("save dss_prj");
        let project = ModProject::load(&prj_path).assert("dss-project");
        project
            .update(&UpdateOptions::default())
            .await
            .assert("spec.update_local");

        project
            .localize(None, LocalizeOptions::for_test())
            .await
            .assert("spec.localize");
        Ok(())
    }
}
