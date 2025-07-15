use crate::const_vars::{VALUE_DIR, VALUE_FILE};
use crate::error::SysReason;
use crate::predule::*;

use crate::{
    const_vars::SYS_MODEL_SPC_ROOT,
    error::SpecResult,
    module::depend::DependencySet,
    types::{Configable, Localizable},
    workflow::prj::GxlProject,
};

use super::{
    init::{SYS_PRJ_ADM, SYS_PRJ_WORK, sys_init_gitignore},
    spec::SysModelSpec,
};
use crate::types::{LocalizeOptions, ValueConfable};
use async_trait::async_trait;
use orion_infra::auto_exit_log;
use orion_x::path::{ensure_path, make_clean_path};
use orion_x::saveable::Persistable;
use orion_x::types::ValuePath;
use orion_x::update::UpdateOptions;
use orion_x::vars::{ValueDict, ValueType};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
struct SysConf {
    test_envs: DependencySet,
}

#[derive(Getters, Clone, Debug)]
pub struct SysProject {
    conf: SysConf,
    sys_spec: SysModelSpec,
    project: GxlProject,
    root_local: PathBuf,
    val_dict: ValueDict,
}
impl SysConf {
    pub fn new(local_res: DependencySet) -> Self {
        Self {
            test_envs: local_res,
        }
    }
}
impl SysProject {
    pub fn new(spec: SysModelSpec, local_res: DependencySet, root_local: PathBuf) -> Self {
        let conf = SysConf::new(local_res);
        let mut val_dict = ValueDict::default();
        val_dict.insert("TEST_WORK_ROOT", ValueType::from("/home/galaxy"));
        Self {
            conf,
            sys_spec: spec,
            project: GxlProject::from((SYS_PRJ_WORK, SYS_PRJ_ADM)),
            root_local,
            val_dict,
        }
    }
    pub fn load(root_local: &Path) -> SpecResult<Self> {
        let mut flag = auto_exit_log!(
            info!(
                target : "ops-prj",
                "load project from {} success!", root_local.display()
            ),
            error!(
                target : "ops-prj",
                "load project  from {} fail!", root_local.display()
            )
        );

        let conf_file = root_local.join("sys_prj.yml");
        let conf = SysConf::from_conf(&conf_file)?;
        let root_local = root_local.to_path_buf();
        let sys_local = root_local.join("sys");
        let sys_spec = SysModelSpec::load_from(&sys_local)?;
        let project = GxlProject::load_from(&root_local).owe(SysReason::Load.into())?;
        let value_root = ensure_path(root_local.join(VALUE_DIR)).owe_logic()?;
        let value_file = value_root.join(VALUE_FILE);
        let val_dict = if value_file.exists() {
            ValueDict::from_conf(&value_file)?
        } else {
            ValueDict::new()
        };
        flag.mark_suc();
        Ok(Self {
            conf,
            sys_spec,
            project,
            root_local,
            val_dict,
        })
    }
    pub fn save(&self) -> SpecResult<()> {
        let mut flag = auto_exit_log!(
            info!(
                target : "sysprj",
                "save project to {} success!", self.root_local().display()
            ),
            error!(
                target : "sysprj",
                "save project  to {} fail!", self.root_local().display()
            )
        );
        let conf_file = self.root_local().join("sys_prj.yml");
        self.conf.save_conf(&conf_file)?;
        self.sys_spec.save_local(self.root_local(), "sys")?;
        self.project
            .save_to(self.root_local(), None)
            .owe(SysReason::Save.into())?;

        let value_root = ensure_path(self.root_local().join(VALUE_DIR)).owe_logic()?;
        let value_file = value_root.join(VALUE_FILE);
        self.val_dict.save_conf(&value_file)?;
        sys_init_gitignore(self.root_local())?;
        flag.mark_suc();
        Ok(())
    }
}

impl SysConf {
    pub async fn update(&self, options: &UpdateOptions) -> SpecResult<()> {
        self.test_envs
            .update(options)
            .await
            .owe(SysReason::Update.into())
    }
}

impl SysProject {
    pub async fn update(&self, options: &UpdateOptions) -> SpecResult<()> {
        self.conf.update(options).await?;
        self.sys_spec().update_local(options).await
    }
}

#[async_trait]
impl Localizable for SysConf {
    async fn localize(
        &self,
        _dst_path: Option<ValuePath>,
        _options: LocalizeOptions,
    ) -> SpecResult<()> {
        Ok(())
    }
}

impl SysProject {
    pub async fn localize(&self, options: LocalizeOptions) -> SpecResult<()> {
        let value_path = self.value_path().ensure_exist().owe_res()?;
        let value_file = value_path.value_file();
        let dict = ValueDict::from_valconf(&value_file)?;
        let cur_opt = options.with_global(dict);
        let dst_path = Some(value_path);

        self.conf
            .localize(dst_path.clone(), cur_opt.clone())
            .await?;
        self.sys_spec().localize(dst_path, cur_opt).await?;
        Ok(())
    }
    pub fn value_path(&self) -> ValuePath {
        let value_root = self.root_local().join(VALUE_DIR);
        ValuePath::from_root(value_root)
    }
}
impl SysProject {
    pub fn make_new(prj_path: &Path, name: &str, repo: &str) -> SpecResult<Self> {
        let mod_spec = SysModelSpec::make_new(name, repo)?;
        let res = DependencySet::default();
        Ok(SysProject::new(mod_spec, res, prj_path.to_path_buf()))
    }
    pub fn make_test_prj(name: &str, repo: &str) -> SpecResult<Self> {
        let prj_path = PathBuf::from(SYS_MODEL_SPC_ROOT).join(name);
        make_clean_path(&prj_path).owe_logic()?;
        let proj = SysProject::make_new(&prj_path, name, repo)?;
        proj.save()?;
        Ok(proj)
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::{Path, PathBuf};

    use orion_error::{ErrorOwe, TestAssertWithMsg};
    use orion_x::{
        addr::{AddrType, GitAddr, types::EnvVarPath},
        path::make_clean_path,
        tools::test_init,
        update::UpdateOptions,
    };

    use crate::{
        const_vars::SYS_MODEL_PRJ_ROOT,
        error::SpecResult,
        module::depend::{Dependency, DependencySet},
        system::{proj::SysProject, spec::SysModelSpec},
        types::LocalizeOptions,
    };
    #[tokio::test]
    async fn test_mod_prj_new() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("sys_new");
        make_clean_path(&prj_path).owe_logic()?;
        let proj = SysProject::make_new(&prj_path, "sys_new", "https://github.com")?;
        proj.save()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sys_prj_example() -> SpecResult<()> {
        test_init();

        let prj_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("example_sys2");
        make_clean_path(&prj_path).owe_logic()?;
        let project = make_sys_prj_testins(&prj_path).assert("make cust");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("ok");
        }
        std::fs::create_dir_all(&prj_path).assert("yes");
        project.save().assert("save dss_prj");
        let project = SysProject::load(&prj_path).assert("dss-project");
        project
            .update(&UpdateOptions::default())
            .await
            .assert("spec.update_local");
        project
            .localize(LocalizeOptions::for_test())
            .await
            .assert("spec.localize");
        Ok(())
    }

    fn make_sys_prj_testins(prj_path: &Path) -> SpecResult<SysProject> {
        let mod_spec = SysModelSpec::for_example("exmaple_sys2")?;
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
        Ok(SysProject::new(mod_spec, res, prj_path.to_path_buf()))
    }
}
