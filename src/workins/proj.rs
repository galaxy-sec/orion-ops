use crate::addr::LocalAddr;
use crate::const_vars::{VALUE_DIR, VALUE_FILE, WORKINS_PRJ_ROOT};
use crate::predule::*;

use crate::system::refs::SysModelSpecRef;
use crate::tools::ensure_path;
use crate::{
    error::SpecResult,
    module::depend::DependencySet,
    tools::make_clean_path,
    types::{Configable, Localizable, Persistable, ValuePath},
    vars::{ValueDict, ValueType},
    workflow::prj::GxlProject,
};
pub const WORKINS_PRJ_WORK: &str = include_str!("init/_gal/work.gxl");
pub const WORKINS_PRJ_ADM: &str = include_str!("init/_gal/adm.gxl");
pub const WORKINS_PRJ_FILE: &str = "workins_prj.yml";

use crate::types::{LocalizeOptions, SysUpdateable, ValueConfable};
use async_trait::async_trait;

use super::init::workins_init_gitignore;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
struct WorkInsConf {
    systems: Vec<SysModelSpecRef>,
    work_envs: DependencySet,
}

impl WorkInsConf {
    pub fn new(local_res: DependencySet) -> Self {
        Self {
            work_envs: local_res,
            systems: Vec::new(),
        }
    }
    pub fn for_test() -> Self {
        let systems = vec![SysModelSpecRef::from(
            "example_sys",
            LocalAddr::from("./example/sys-model-spec/example_sys"),
        )];
        let work_envs = DependencySet::example();
        Self { systems, work_envs }
    }
}

#[derive(Getters, Clone, Debug)]
pub struct WorkInsProject {
    conf: WorkInsConf,
    project: GxlProject,
    root_local: PathBuf,
    val_dict: ValueDict,
}
impl WorkInsProject {
    pub fn new(conf: WorkInsConf, root_local: PathBuf) -> Self {
        let mut val_dict = ValueDict::default();
        val_dict.insert("TEST_WORK_ROOT", ValueType::from("/home/galaxy"));
        Self {
            conf,
            project: GxlProject::from((WORKINS_PRJ_WORK, WORKINS_PRJ_ADM)),
            root_local,
            val_dict,
        }
    }
    pub fn load(root_local: &Path) -> SpecResult<Self> {
        let mut flag = log_guard!(
            info!(
                target : "/sys_prj",
                "load project from {} success!", root_local.display()
            ),
            error!(
                target : "/sys_prj",
                "load project  from {} fail!", root_local.display()
            )
        );

        let conf_file = root_local.join(WORKINS_PRJ_FILE);
        let conf = WorkInsConf::from_conf(&conf_file)?;
        let root_local = root_local.to_path_buf();
        let project = GxlProject::load_from(&root_local)?;
        let value_root = ensure_path(root_local.join(VALUE_DIR))?;
        let value_file = value_root.join(VALUE_FILE);
        let val_dict = if value_file.exists() {
            ValueDict::from_conf(&value_file)?
        } else {
            ValueDict::new()
        };
        flag.flag_suc();
        Ok(Self {
            conf,
            project,
            root_local,
            val_dict,
        })
    }
    pub fn save(&self) -> SpecResult<()> {
        let mut flag = log_guard!(
            info!(
                target : "sysprj",
                "save project to {} success!", self.root_local().display()
            ),
            error!(
                target : "sysprj",
                "save project  to {} fail!", self.root_local().display()
            )
        );
        let conf_file = self.root_local().join(WORKINS_PRJ_FILE);
        self.conf.save_conf(&conf_file)?;
        self.project.save_to(self.root_local(), None)?;

        let value_root = ensure_path(self.root_local().join(VALUE_DIR))?;
        let value_file = value_root.join(VALUE_FILE);
        self.val_dict.save_conf(&value_file)?;
        workins_init_gitignore(self.root_local())?;
        flag.flag_suc();
        Ok(())
    }
}

#[async_trait]
impl SysUpdateable<WorkInsConf> for WorkInsConf {
    async fn update_local(mut self, path: &Path, options: &UpdateOptions) -> SpecResult<Self> {
        self.work_envs.update(options).await?;
        let mut updated_sys = Vec::new();
        for sys in self.systems {
            updated_sys.push(sys.update_local(path, options).await?);
        }
        self.systems = updated_sys;
        Ok(self)
    }
}

#[async_trait]
impl SysUpdateable<WorkInsProject> for WorkInsProject {
    async fn update_local(mut self, path: &Path, options: &UpdateOptions) -> SpecResult<Self> {
        self.conf = self.conf.update_local(path, options).await?;
        Ok(self)
    }
}

impl WorkInsProject {
    pub async fn update(self, options: &UpdateOptions) -> SpecResult<Self> {
        let path = self.root_local().clone();
        self.update_local(&path, options).await
    }
}

#[async_trait]
impl Localizable for WorkInsConf {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        for sys in self.systems() {
            sys.localize(dst_path.clone(), options.clone()).await?;
        }
        Ok(())
    }
}

impl WorkInsProject {
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
impl WorkInsProject {
    pub fn make_new(prj_path: &Path, name: &str) -> SpecResult<Self> {
        let conf = WorkInsConf::for_test();
        Ok(WorkInsProject::new(conf, prj_path.to_path_buf()))
    }
    pub fn for_test(name: &str) -> SpecResult<Self> {
        let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join(name);
        make_clean_path(&prj_path)?;
        let proj = WorkInsProject::make_new(&prj_path, name)?;
        proj.save()?;
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
        tools::{make_clean_path, test_init},
        types::LocalizeOptions,
        update::UpdateOptions,
        workins::proj::WorkInsProject,
    };

    #[tokio::test]
    async fn test_workins_example() -> SpecResult<()> {
        test_init();
        let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join("workins_sys_1");
        make_clean_path(&prj_path)?;
        let project = WorkInsProject::for_test("workins_sys_1".into()).assert("make workins");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("ok");
        }
        std::fs::create_dir_all(&prj_path).assert("yes");
        project.save().assert("save workins_prj");
        let mut project = WorkInsProject::load(&prj_path).assert("workins-prj");
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
