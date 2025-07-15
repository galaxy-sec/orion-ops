use crate::{predule::*, types::Yamlable};
use std::path::{Path, PathBuf};

use crate::{
    const_vars::{MOD_LIST_YML, MODULES_SPC_ROOT, NET_RES_YML, RESOURCE_YML, VARS_YML},
    error::ElementReason,
    module::proj::ModProject,
    types::Localizable,
    workflow::act::SysWorkflows,
};
use async_trait::async_trait;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};
use orion_infra::auto_exit_log;
use orion_x::{
    addr::{GitAddr, LocalAddr},
    types::ValuePath,
    update::UpdateOptions,
};

use super::{
    ModulesList,
    init::{SysIniter, sys_init_gitignore},
};
use crate::types::LocalizeOptions;
use crate::{
    error::{SpecReason, SpecResult, ToErr},
    module::{CpuArch, ModelSTD, OsCPE, RunSPC, refs::ModuleSpecRef, spec::ModuleSpec},
    types::Configable,
};
#[derive(Getters, Clone, Debug)]
pub struct SysModelSpec {
    name: String,
    mod_list: ModulesList,
    local: Option<PathBuf>,
    workflow: SysWorkflows,
}

#[derive(Getters, Clone, Debug)]
pub struct SysTargetPaths {
    target_root: PathBuf,
    spec_path: PathBuf,
    net_path: PathBuf,
    res_path: PathBuf,
    vars_path: PathBuf,
    modlist_path: PathBuf,
    workflow_path: PathBuf,
}
impl From<&PathBuf> for SysTargetPaths {
    fn from(target_root: &PathBuf) -> Self {
        //let spec_path = target_root.join(SPEC_DIR);
        Self {
            target_root: target_root.to_path_buf(),
            net_path: target_root.join(NET_RES_YML),
            res_path: target_root.join(RESOURCE_YML),
            vars_path: target_root.join(VARS_YML),
            modlist_path: target_root.join(MOD_LIST_YML),
            workflow_path: target_root.to_path_buf(),
            spec_path: target_root.clone(),
        }
    }
}
impl SysModelSpec {
    pub fn add_mod(&mut self, modx: ModuleSpec) {
        self.mod_list.add_mod(modx);
    }
    pub fn add_mod_ref(&mut self, modx: ModuleSpecRef) {
        self.mod_list.add_ref(modx)
    }
    pub fn save_to(&self, path: &Path) -> SpecResult<()> {
        self.save_local(path, self.name())
    }
    pub fn save_local(&self, path: &Path, name: &str) -> SpecResult<()> {
        let root = path.join(name);

        let mut flag = auto_exit_log!(
            info!(target: "sys", "save sys spec success!:{}", root.display()),
            error!(target: "sys", "save sys spec failed!:{}", root.display())
        );
        let paths = SysTargetPaths::from(&root);
        std::fs::create_dir_all(paths.spec_path()).owe_conf()?;
        sys_init_gitignore(&root)?;
        self.mod_list.save_conf(paths.modlist_path())?;

        self.workflow.save_to(paths.workflow_path(), None)?;
        flag.mark_suc();
        Ok(())
    }

    pub fn load_from(root: &Path) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load syspec");
        let name = root
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad name".to_string()))?;

        let mut flag = auto_exit_log!(
            info!(target: "sys", "load sys spec success!:{}", root.display()),
            error!(target: "sys", "load sys spec failed!:{}", root.display())
        );
        let paths = SysTargetPaths::from(&root.to_path_buf());

        ctx.with_path("mod_list", paths.modlist_path());
        let mut mod_list = ModulesList::from_conf(paths.modlist_path())
            .with("load mod-list".to_string())
            .with(&ctx)?;
        mod_list.set_mods_local(paths.spec_path().clone());
        let workflow = SysWorkflows::load_from(paths.workflow_path()).with(&ctx)?;
        flag.mark_suc();
        Ok(Self {
            name: name.to_string(),
            mod_list,
            local: Some(root.to_path_buf()),
            workflow,
        })
    }

    pub fn new<S: Into<String>>(name: S, actions: SysWorkflows) -> Self {
        Self {
            name: name.into(),
            mod_list: ModulesList::default(),
            local: None,
            workflow: actions,
        }
    }

    pub async fn update_local(&self, options: &UpdateOptions) -> SpecResult<()> {
        if let Some(local) = &self.local {
            let value = self.mod_list.update(local, options).await?;
            let path = local.join("vars.yml");
            if path.exists() {
                std::fs::remove_file(&path).owe_sys()?;
            }
            value.vars.save_yml(&path)?;
            Ok(())
        } else {
            SpecReason::from(ElementReason::Miss("local path".into())).err_result()
        }
    }
}

#[async_trait]
impl Localizable for SysModelSpec {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        if let Some(_local) = &self.local {
            self.mod_list.localize(dst_path, options).await?;
            Ok(())
        } else {
            SpecReason::from(ElementReason::Miss("local path".into())).err_result()
        }
    }
}
impl SysModelSpec {
    pub fn for_example(name: &str) -> SpecResult<SysModelSpec> {
        ModProject::make_test_prj("redis2_mock")?;
        ModProject::make_test_prj("mysql2_mock")?;
        make_sys_spec_test(name, vec!["redis2_mock", "mysql2_mock"])
    }

    pub fn make_new(name: &str, repo: &str) -> SpecResult<SysModelSpec> {
        let actions = SysWorkflows::sys_tpl_init();
        let mut modul_spec = SysModelSpec::new(name, actions);
        let mod_name = "you_mod1";

        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                mod_name,
                GitAddr::from(repo).with_tag("0.1.0"),
                ModelSTD::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
            )
            .with_enable(false),
        );
        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                "you_mod2",
                GitAddr::from(repo).with_branch("beta"),
                ModelSTD::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
            )
            .with_enable(false),
        );
        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                "you_mod3",
                GitAddr::from("http://github").with_tag("v1.0.0"),
                ModelSTD::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
            )
            .with_enable(false),
        );
        Ok(modul_spec)
    }
}

pub fn make_sys_spec_test(name: &str, mod_names: Vec<&str>) -> SpecResult<SysModelSpec> {
    let actions = SysWorkflows::sys_tpl_init();
    let mut modul_spec = SysModelSpec::new(name, actions);
    for mod_name in mod_names {
        //let mod_name = "postgresql";
        modul_spec.add_mod_ref(ModuleSpecRef::from(
            mod_name,
            LocalAddr::from(format!("{}/{}", MODULES_SPC_ROOT, mod_name)),
            ModelSTD::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        ));
    }

    Ok(modul_spec)
}

#[cfg(test)]
pub mod tests {

    use orion_error::TestAssertWithMsg;
    use orion_x::{path::make_clean_path, tools::test_init};

    use crate::{const_vars::SYS_MODEL_SPC_ROOT, module::proj::ModProject};

    use super::*;

    #[tokio::test]
    async fn build_example_sys_spec() -> SpecResult<()> {
        test_init();
        let sys_name = "example_sys";
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT).join(sys_name);
        make_clean_path(&spec_root)?;
        ModProject::make_test_prj("redis_mock")?;
        ModProject::make_test_prj("mysql_mock")?;
        let spec =
            make_sys_spec_test(sys_name, vec!["redis_mock", "mysql_mock"]).assert("make spec");
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT);
        let spec_path = spec_root.join(spec.name());
        make_clean_path(&spec_path)?;
        spec.save_to(&spec_root).assert("spec save");
        let spec = SysModelSpec::load_from(&spec_root.join(spec.name())).assert("spec load");
        spec.update_local(&UpdateOptions::for_test())
            .await
            .assert("update");
        spec.localize(None, LocalizeOptions::for_test())
            .await
            .assert("localize");
        Ok(())
    }
}
