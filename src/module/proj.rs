use orion_variate::addr::HttpResource;

use super::prelude::*;
use crate::const_vars::{
    BITNAMI_COMMON_GIT_URL, MOD_PRJ_CONF_FILE_V1, MOD_PRJ_CONF_FILE_V2, MOD_PRJ_TEST_ROOT,
};
use crate::error::ModReason;
use crate::module::init::MOD_PRJ_ROOT_FILE;
use crate::predule::*;
use crate::types::{Localizable, RefUpdateable, ValuePath};

use super::init::{MOD_PRJ_ADM_GXL, MOD_PRJ_WORK_GXL, mod_init_gitignore};
use crate::{
    const_vars::MODULES_SPC_ROOT,
    module::{
        depend::{Dependency, DependencySet},
        spec::ModuleSpec,
    },
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
        val_dict.insert("TEST_WORK_ROOT", ValueType::from(MOD_PRJ_TEST_ROOT));
        Self {
            conf,
            mod_spec: spec,
            project: GxlProject::from((MOD_PRJ_WORK_GXL, MOD_PRJ_ADM_GXL, MOD_PRJ_ROOT_FILE)),
            root_local,
        }
    }
    pub fn load(root_local: &Path) -> MainResult<Self> {
        let mut flag = auto_exit_log!(
            info!(
                target : "/mod_prj",
                 "load mod-prj  to {} success!", root_local.display()
            ),
            error!(
                target : "/mod_prj",
                "load mod-prj  to {} fail!", root_local.display()
            )
        );

        let conf_file_v1 = root_local.join(MOD_PRJ_CONF_FILE_V1);
        let conf_file_v2 = root_local.join(MOD_PRJ_CONF_FILE_V2);
        if conf_file_v1.exists() {
            std::fs::rename(&conf_file_v1, &conf_file_v2).owe_res()?;
        };
        let conf = ModConf::from_conf(&conf_file_v2).owe_logic()?;
        let root_local = root_local.to_path_buf();
        let mod_spec = ModuleSpec::load_from(&root_local).owe(ModReason::Load.into())?;
        let project = GxlProject::load_from(&root_local).owe(ModReason::Load.into())?;
        flag.mark_suc();
        Ok(Self {
            conf,
            mod_spec,
            project,
            root_local,
        })
    }
    pub fn save(&self) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(
                target : "spec/local/modprj",
                 "save modprj  to {} success!", self.root_local().display()
            ),
            error!(
               target : "spec/local/modprj",
               "save modprj  to {} fail!", self.root_local().display()
            )
        );
        let conf_file = self.root_local().join("mod-prj.yml");
        self.conf.save_conf(&conf_file).owe_res()?;
        self.mod_spec
            .save_to(self.root_local(), Some("./".into()))
            .owe(ModReason::Save.into())?;
        self.project
            .save_to(self.root_local(), None)
            .owe(ModReason::Save.into())?;
        mod_init_gitignore(self.root_local())?;
        flag.mark_suc();
        Ok(())
    }
}

#[async_trait]
impl RefUpdateable<()> for ModConf {
    async fn update_local(
        &self,
        accessor: Accessor,
        _path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<()> {
        self.test_envs
            .update_local(accessor, _path, options)
            .await
            .owe(ModReason::Update.into())
    }
}

#[async_trait]
impl RefUpdateable<()> for ModProject {
    async fn update_local(
        &self,
        accessor: Accessor,
        _path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<()> {
        self.conf
            .update_local(accessor.clone(), _path, options)
            .await?;
        self.mod_spec()
            .update_local(accessor, self.root_local(), options)
            .await
            .owe(ModReason::Update.into())?;
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModConf {
    async fn localize(
        &self,
        _dst_path: Option<ValuePath>,
        _options: LocalizeOptions,
    ) -> MainResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModProject {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> MainResult<()> {
        //let local_path = LocalizePath::from_root(self.root_local());
        self.conf
            .localize(dst_path.clone(), options.clone())
            .await?;
        self.mod_spec().localize(dst_path, options).await?;
        Ok(())
    }
}
impl ModProject {
    pub fn make_new(prj_path: &Path, name: &str) -> MainResult<Self> {
        let mod_spec = ModuleSpec::make_new(name)?;
        let res = DependencySet::default();
        Ok(ModProject::new(mod_spec, res, prj_path.to_path_buf()))
    }
    pub fn make_test_prj(name: &str) -> MainResult<Self> {
        let prj_path = PathBuf::from(MODULES_SPC_ROOT).join(name);
        make_clean_path(&prj_path).owe_logic()?;
        let proj = ModProject::make_new(&prj_path, name)?;
        proj.save()?;
        Ok(proj)
    }
}

pub fn make_mod_prj_testins(prj_path: &Path) -> MainResult<ModProject> {
    let mod_spec = ModuleSpec::for_example();
    let mut res = DependencySet::default();
    res.push(
        Dependency::new(
            Address::from(HttpResource::from(BITNAMI_COMMON_GIT_URL)),
            PathTemplate::from(prj_path.join("test_res")),
        )
        .with_rename("bit-common"),
    );
    Ok(ModProject::new(mod_spec, res, prj_path.to_path_buf()))
}

#[cfg(test)]
pub mod tests {
    use crate::{
        accessor::accessor_for_test,
        predule::*,
        types::{LocalizeOptions, RefUpdateable},
    };
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;
    use orion_infra::path::make_clean_path;
    use orion_variate::{tools::test_init, update::DownloadOptions};

    use crate::{
        const_vars::MODULES_SPC_ROOT,
        module::proj::{ModProject, make_mod_prj_testins},
        types::Localizable,
    };
    #[tokio::test]
    async fn test_mod_prj_new() -> MainResult<()> {
        test_init();
        let prj_path = PathBuf::from(MODULES_SPC_ROOT).join("mod-new");
        make_clean_path(&prj_path).owe_logic()?;
        let proj = ModProject::make_new(&prj_path, "mod_new")?;
        proj.save()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mod_prj_example() -> MainResult<()> {
        test_init();

        let prj_path = PathBuf::from(MODULES_SPC_ROOT).join("postgresql");
        let project = make_mod_prj_testins(&prj_path).assert("make cust");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("ok");
        }
        std::fs::create_dir_all(&prj_path).assert("yes");
        project.save().assert("save dss_prj");
        let project = ModProject::load(&prj_path).assert("dss-project");
        let accessor = accessor_for_test();
        project
            .update_local(accessor, &prj_path, &DownloadOptions::default())
            .await
            .assert("spec.update_local");

        project
            .localize(None, LocalizeOptions::for_test())
            .await
            .assert("spec.localize");
        Ok(())
    }
}
