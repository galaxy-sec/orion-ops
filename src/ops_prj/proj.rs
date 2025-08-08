use crate::const_vars::{VALUE_DIR, VALUE_FILE, WORKINS_PRJ_ROOT};
use crate::error::OpsReason;
use crate::ops_prj::system::{OpsSystem, OpsTarget};
use crate::predule::*;

use crate::{error::MainResult, module::depend::DependencySet, workflow::prj::GxlProject};
const OPS_PRJ_WORK: &str = include_str!("init/_gal/work.gxl");
const OPS_PRJ_ADM: &str = include_str!("init/_gal/adm.gxl");
const OPS_PRJ_FILE: &str = "ops-prj.yml";
const PRJ_OPS_TARGET: &str = "ops-systems.yml";

use crate::types::{Accessor, InsUpdateable, ValuePath};
use async_trait::async_trait;
use getset::MutGetters;
use orion_common::serde::{Configable, Persistable};
use orion_infra::auto_exit_log;
use orion_infra::path::{ensure_path, make_clean_path};
use orion_variate::update::DownloadOptions;
use orion_variate::vars::{ValueDict, ValueType};

use super::conf::ProjectConf;
use super::init::workins_init_gitignore;

#[derive(Getters, Clone, Debug, MutGetters)]
pub struct OpsProject {
    conf: ProjectConf,
    project: GxlProject,
    root_local: PathBuf,
    val_dict: ValueDict,
    #[getset(get = "pub", get_mut = "pub")]
    ops_target: OpsTarget,
}
impl OpsProject {
    pub fn new(conf: ProjectConf, root_local: PathBuf) -> Self {
        let mut val_dict = ValueDict::default();
        val_dict.insert("TEST_WORK_ROOT", ValueType::from("/home/galaxy"));
        Self {
            conf,
            project: GxlProject::from((OPS_PRJ_WORK, OPS_PRJ_ADM)),
            root_local,
            val_dict,
            ops_target: OpsTarget::default(),
        }
    }
    pub fn import_ops_sys(&mut self, ops_sys: OpsSystem) {
        if !self.ops_target.contains(&ops_sys) {
            self.ops_target.push(ops_sys);
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

        let conf = ProjectConf::load(root_local)?;
        let os_target_path = root_local.join(PRJ_OPS_TARGET);
        let ops_target = OpsTarget::from_conf(&os_target_path).owe_conf()?;
        let root_local = root_local.to_path_buf();
        let project = GxlProject::load_from(&root_local).owe(OpsReason::Load.into())?;
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
            project,
            root_local,
            val_dict,
            ops_target,
        })
    }
    pub fn save(&self) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(
                target : "workprj",
                "save project to {} success!", self.root_local().display()
            ),
            error!(
                target : "workprj",
                "save project  to {} fail!", self.root_local().display()
            )
        );
        let conf_file = self.root_local().join(OPS_PRJ_FILE);
        let os_target_path = self.root_local().join(PRJ_OPS_TARGET);
        self.ops_target.save_conf(&os_target_path).owe_res()?;
        self.conf.save_conf(&conf_file).owe_res()?;
        self.project.save_to(self.root_local(), None).owe_logic()?;

        let value_root = ensure_path(self.root_local().join(VALUE_DIR)).owe_logic()?;
        let value_file = value_root.join(VALUE_FILE);
        self.val_dict.save_conf(&value_file).owe_res()?;
        workins_init_gitignore(self.root_local())?;
        flag.mark_suc();
        Ok(())
    }
}

#[async_trait]
impl InsUpdateable<OpsProject> for OpsProject {
    async fn update_local(
        mut self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<Self> {
        self.conf = self.conf.update_local(accessor, path, options).await?;
        self.save()?;
        Ok(self)
    }
}

impl OpsProject {
    pub fn value_path(&self) -> ValuePath {
        let value_root = self.root_local().join(VALUE_DIR);
        ValuePath::from_root(value_root)
    }
}
impl OpsProject {
    pub fn make_new(prj_path: &Path, name: &str) -> MainResult<Self> {
        let conf = ProjectConf::new(name, DependencySet::default());
        Ok(OpsProject::new(conf, prj_path.to_path_buf()))
    }
    pub fn for_test(name: &str) -> MainResult<Self> {
        let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join(name);
        make_clean_path(&prj_path).owe_logic()?;

        let conf = ProjectConf::for_test();
        let proj = OpsProject::new(conf, prj_path.to_path_buf());
        Ok(proj)
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::{ErrorOwe, TestAssertWithMsg};
    use orion_infra::path::make_clean_path;
    use orion_variate::{tools::test_init, update::DownloadOptions};

    use crate::{
        accessor::accessor_for_test, const_vars::WORKINS_PRJ_ROOT, error::MainResult,
        ops_prj::proj::OpsProject, types::InsUpdateable,
    };

    #[tokio::test]
    async fn test_workins_example() -> MainResult<()> {
        test_init();
        let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join("workins_sys_1");
        make_clean_path(&prj_path).owe_logic()?;
        let project = OpsProject::for_test("workins_sys_1").assert("make workins");
        project.save().assert("save workins_prj");
        let project = OpsProject::load(&prj_path).assert("workins-prj");
        let accessor = accessor_for_test();
        project
            .update_local(accessor, &prj_path, &DownloadOptions::default())
            .await
            .assert("spec.update_local");
        Ok(())
    }
}
