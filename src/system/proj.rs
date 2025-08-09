use crate::const_vars::{SYS_PRJ_CONF_FILE_V1, SYS_PRJ_CONF_FILE_V2, VALUE_DIR, VALUE_FILE};
use crate::error::SysReason;
use crate::module::ModelSTD;
use crate::predule::*;

use crate::system::spec::SysDefine;
use crate::{
    const_vars::SYS_MODEL_SPC_ROOT, error::MainResult, module::depend::DependencySet,
    types::Localizable, workflow::prj::GxlProject,
};

use super::conf::SysConf;
use super::{
    init::{SYS_PRJ_ADM, SYS_PRJ_WORK, sys_init_gitignore},
    spec::SysModelSpec,
};
use crate::types::{Accessor, LocalizeOptions, RefUpdateable, ValuePath};
use async_trait::async_trait;
use orion_common::serde::{Configable, Persistable};
use orion_infra::auto_exit_log;
use orion_infra::path::{ensure_path, make_clean_path};
use orion_variate::update::DownloadOptions;
use orion_variate::vars::{ValueDict, ValueType};

#[derive(Getters, Clone, Debug)]
pub struct SysProject {
    conf: SysConf,
    sys_spec: SysModelSpec,
    project: GxlProject,
    root_local: PathBuf,
    val_dict: ValueDict,
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
    pub fn load(root_local: &Path) -> MainResult<Self> {
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

        let conf_file_v1 = root_local.join(SYS_PRJ_CONF_FILE_V1);
        let conf_file_v2 = root_local.join(SYS_PRJ_CONF_FILE_V2);
        if conf_file_v1.exists() {
            std::fs::rename(&conf_file_v1, &conf_file_v2).owe_res()?;
        }
        let conf = SysConf::from_conf(&conf_file_v2).owe_res()?;
        let root_local = root_local.to_path_buf();
        let sys_path = root_local.join("sys");
        let sys_spec = SysModelSpec::load_from(&sys_path)?;
        let project = GxlProject::load_from(&root_local).owe(SysReason::Load.into())?;
        let value_root = ensure_path(root_local.join(VALUE_DIR)).owe_logic()?;
        let value_file = value_root.join(VALUE_FILE);
        let val_dict = if value_file.exists() {
            ValueDict::from_conf(&value_file).owe_data()?
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
    pub fn save(&self) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(
                target : "sysprj",
                "save project to {} success!", self.root_local().display()
            ),
            error!(
                target : "sysprj",
                "save project to {} fail!", self.root_local().display()
            )
        );
        let conf_file_v2 = self.root_local().join("sys-prj.yml");
        self.conf.save_conf(&conf_file_v2).owe_res()?;
        self.sys_spec.save_local(self.root_local(), "sys")?;
        self.project
            .save_to(self.root_local(), None)
            .owe(SysReason::Save.into())?;

        // 保存 sys_local 配置

        let value_root = ensure_path(self.root_local().join(VALUE_DIR)).owe_logic()?;
        let value_file = value_root.join(VALUE_FILE);
        self.val_dict.save_conf(&value_file).owe_res()?;
        sys_init_gitignore(self.root_local())?;
        flag.mark_suc();
        Ok(())
    }
}

#[async_trait]
impl RefUpdateable<()> for SysProject {
    async fn update_local(
        &self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<()> {
        self.conf
            .update_local(accessor.clone(), path, options)
            .await?;
        self.sys_spec().update_local(accessor, path, options).await
    }
}

impl SysProject {
    pub async fn localize(&self, options: LocalizeOptions) -> MainResult<()> {
        let value_path = self.value_path().ensure_exist().owe_res()?;
        let dst_path = Some(value_path);

        self.conf
            .localize(dst_path.clone(), options.clone())
            .await?;
        self.sys_spec()
            .localize(dst_path.clone(), options.clone())
            .await?;
        Ok(())
    }
    pub fn value_path(&self) -> ValuePath {
        let value_root = self.root_local().join(VALUE_DIR);
        ValuePath::from_root(value_root)
    }
}
impl SysProject {
    pub fn make_new(prj_path: &Path, name: &str, model: ModelSTD) -> MainResult<Self> {
        let mod_spec = SysModelSpec::make_new(SysDefine::new(name, model))?;
        let res = DependencySet::default();
        Ok(SysProject::new(mod_spec, res, prj_path.to_path_buf()))
    }
    pub fn make_test_prj(name: &str) -> MainResult<Self> {
        let prj_path = PathBuf::from(SYS_MODEL_SPC_ROOT).join(name);
        make_clean_path(&prj_path).owe_logic()?;
        let proj = SysProject::make_new(&prj_path, name, ModelSTD::from_cur_sys())?;
        proj.save()?;
        Ok(proj)
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::{Path, PathBuf};

    use orion_error::{ErrorOwe, TestAssertWithMsg};
    use orion_infra::path::make_clean_path;
    use orion_variate::{
        addr::{Address, HttpResource, types::PathTemplate},
        tools::test_init,
        update::DownloadOptions,
    };

    use crate::{
        accessor::accessor_for_test,
        const_vars::SYS_MODEL_PRJ_ROOT,
        error::MainResult,
        module::{
            ModelSTD,
            depend::{Dependency, DependencySet},
        },
        system::{proj::SysProject, spec::SysModelSpec},
        types::{LocalizeOptions, RefUpdateable},
    };
    #[tokio::test]
    async fn test_mod_prj_new() -> MainResult<()> {
        test_init();
        let prj_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("sys_new");
        make_clean_path(&prj_path).owe_logic()?;
        let proj = SysProject::make_new(&prj_path, "sys_new", ModelSTD::from_cur_sys())?;
        proj.save()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sys_prj_example() -> MainResult<()> {
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
        let accessor = accessor_for_test();
        project
            .update_local(accessor, &prj_path, &DownloadOptions::default())
            .await
            .assert("spec.update_local");
        project
            .localize(LocalizeOptions::for_test())
            .await
            .assert("spec.localize");
        Ok(())
    }

    fn make_sys_prj_testins(prj_path: &Path) -> MainResult<SysProject> {
        let mod_spec = SysModelSpec::for_example("exmaple_sys2")?;
        let mut res = DependencySet::default();
        res.push(
            Dependency::new(
                Address::from(HttpResource::from(
                    "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
                )),
                PathTemplate::from(prj_path.join("test_res")),
            )
            .with_rename("bit-common"),
        );
        Ok(SysProject::new(mod_spec, res, prj_path.to_path_buf()))
    }
}
