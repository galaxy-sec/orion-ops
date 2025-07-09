use crate::const_vars::{VALUE_DIR, VALUE_FILE, WORKINS_PRJ_ROOT};
use crate::predule::*;

use crate::tools::ensure_path;
use crate::{
    error::SpecResult,
    module::depend::DependencySet,
    tools::make_clean_path,
    types::{Configable, Localizable, Persistable, ValuePath},
    vars::{ValueDict, ValueType},
    workflow::prj::GxlProject,
};
const OPS_PRJ_WORK: &str = include_str!("init/_gal/work.gxl");
const OPS_PRJ_ADM: &str = include_str!("init/_gal/adm.gxl");
const OPS_PRJ_FILE: &str = "ops-prj.yml";

use crate::types::{LocalizeOptions, SysUpdateable, ValueConfable};
use async_trait::async_trait;

use super::conf::ProjectConf;
use super::init::workins_init_gitignore;

#[derive(Getters, Clone, Debug)]
pub struct OpsProject {
    conf: ProjectConf,
    project: GxlProject,
    root_local: PathBuf,
    val_dict: ValueDict,
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

        let conf = ProjectConf::load(root_local)?;
        let root_local = root_local.to_path_buf();
        let project = GxlProject::load_from(&root_local)?;
        let value_root = ensure_path(root_local.join(VALUE_DIR))?;
        let value_file = value_root.join(VALUE_FILE);
        let val_dict = if value_file.exists() {
            ValueDict::from_conf(&value_file)?
        } else {
            ValueDict::new()
        };
        flag.mark_suc();
        Ok(Self {
            conf,
            project,
            root_local,
            val_dict,
        })
    }
    pub fn save(&self) -> SpecResult<()> {
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
        self.conf.save_conf(&conf_file)?;
        self.project.save_to(self.root_local(), None)?;

        let value_root = ensure_path(self.root_local().join(VALUE_DIR))?;
        let value_file = value_root.join(VALUE_FILE);
        self.val_dict.save_conf(&value_file)?;
        workins_init_gitignore(self.root_local())?;
        flag.mark_suc();
        Ok(())
    }
}

#[async_trait]
impl SysUpdateable<OpsProject> for OpsProject {
    async fn update_local(mut self, path: &Path, options: &UpdateOptions) -> SpecResult<Self> {
        self.conf = self.conf.update_local(path, options).await?;
        self.save()?;
        Ok(self)
    }
}

impl OpsProject {
    pub async fn update(self, options: &UpdateOptions) -> SpecResult<Self> {
        let path = self.root_local().clone();
        self.update_local(&path, options).await
    }
}

impl OpsProject {
    pub async fn localize(&self, options: LocalizeOptions) -> SpecResult<()> {
        let value_path = self.value_path().ensure_exist()?;
        let value_file = value_path.value_file();
        let dict = ValueDict::from_valconf(&value_file)?;
        let cur_opt = options.with_global(dict);
        let dst_path = Some(value_path);

        self.conf
            .localize(dst_path.clone(), cur_opt.clone())
            .await?;
        Ok(())
    }
    pub fn value_path(&self) -> ValuePath {
        let value_root = self.root_local().join(VALUE_DIR);
        ValuePath::from_root(value_root)
    }
}
impl OpsProject {
    pub fn make_new(prj_path: &Path, name: &str) -> SpecResult<Self> {
        let conf = ProjectConf::new(name, DependencySet::default());
        Ok(OpsProject::new(conf, prj_path.to_path_buf()))
    }
    pub fn for_test(name: &str) -> SpecResult<Self> {
        let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join(name);
        make_clean_path(&prj_path)?;

        let conf = ProjectConf::for_test();
        let proj = OpsProject::new(conf, prj_path.to_path_buf());
        Ok(proj)
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        const_vars::WORKINS_PRJ_ROOT,
        error::SpecResult,
        ops_prj::proj::OpsProject,
        tools::{make_clean_path, test_init},
        types::LocalizeOptions,
        update::UpdateOptions,
    };

    #[tokio::test]
    async fn test_workins_example() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join("workins_sys_1");
        make_clean_path(&prj_path)?;
        let project = OpsProject::for_test("workins_sys_1".into()).assert("make workins");
        project.save().assert("save workins_prj");
        let mut project = OpsProject::load(&prj_path).assert("workins-prj");
        project = project
            .update(&UpdateOptions::default())
            .await
            .assert("spec.update_local");
        project
            .localize(LocalizeOptions::for_test())
            .await
            .assert("spec.localize");
        Ok(())
    }
}
