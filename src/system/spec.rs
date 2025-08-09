use crate::{
    error::{MainError, SysReason},
    local::LocalizeVarPath,
    predule::*,
    system::path::SysTargetPaths,
    types::{Accessor, RefUpdateable, ValuePath},
};
use std::path::{Path, PathBuf};

use crate::{
    const_vars::MODULES_SPC_ROOT, error::ElementReason, module::proj::ModProject,
    types::Localizable, workflow::act::SysWorkflows,
};
use async_trait::async_trait;
use getset::{Getters, WithSetters};
use orion_common::serde::{Configable, Persistable, Yamlable};
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, UvsLogicFrom, WithContext};
use orion_infra::auto_exit_log;
use orion_variate::{
    addr::{GitRepository, LocalPath},
    update::DownloadOptions,
};

use super::{
    ModulesList,
    init::{SysIniter, sys_init_gitignore},
};
use crate::types::LocalizeOptions;
use crate::{
    error::{MainReason, MainResult, ToErr},
    module::{CpuArch, ModelSTD, OsCPE, RunSPC, refs::ModuleSpecRef, spec::ModuleSpec},
};

#[derive(Clone, Debug, Serialize, Deserialize, Getters, WithSetters, PartialEq)]
#[getset(get = "pub ")]
pub struct SysDefine {
    name: String,
    model: ModelSTD,
    #[getset(set_with = "pub ")]
    vender: String,
}
impl SysDefine {
    pub fn new<S: Into<String>>(name: S, model: ModelSTD) -> Self {
        Self {
            name: name.into(),
            vender: String::new(),
            model,
        }
    }
}
#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub ")]
pub struct SysModelSpec {
    define: SysDefine,
    mod_list: ModulesList,
    local: Option<PathBuf>,
    #[serde(skip)]
    workflow: SysWorkflows,
}

impl SysModelSpec {
    pub fn add_mod(&mut self, modx: ModuleSpec) {
        self.mod_list.add_mod(modx);
    }
    pub fn add_mod_ref(&mut self, modx: ModuleSpecRef) {
        self.mod_list.add_ref(modx)
    }
    pub fn save_to(&self, path: &Path) -> MainResult<()> {
        self.save_local(path, self.define.name())
    }
    pub fn save_local(&self, path: &Path, name: &str) -> MainResult<()> {
        let root = path.join(name);

        let mut flag = auto_exit_log!(
            info!(target: "sys", "save sys spec success!:{}", root.display()),
            error!(target: "sys", "save sys spec failed!:{}", root.display())
        );
        let paths = SysTargetPaths::from(&root);
        std::fs::create_dir_all(paths.spec_path()).owe_conf()?;
        sys_init_gitignore(&root)?;
        self.define.save_conf(paths.define_path()).owe_res()?;
        self.mod_list.save_conf(paths.modlist_path()).owe_res()?;

        self.workflow
            .save_to(paths.workflow_path(), None)
            .owe_logic()?;
        flag.mark_suc();
        Ok(())
    }

    pub fn load_from(root: &Path) -> MainResult<Self> {
        let mut ctx = WithContext::want("load syspec");
        let _name = root
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad name".to_string()))?;

        let mut flag = auto_exit_log!(
            info!(target: "sys", "load sys spec success!:{}", root.display()),
            error!(target: "sys", "load sys spec failed!:{}", root.display())
        );
        let paths = SysTargetPaths::from(&root.to_path_buf());

        ctx.with_path("mod_list", paths.modlist_path());
        let define = if !paths.define_path().exists() {
            return MainError::from_logic(format!(
                "miss define file : {}",
                paths.define_path().display()
            ))
            .err();
        } else {
            SysDefine::from_conf(paths.define_path())
                .with("load define".to_string())
                .with(&ctx)
                .owe_data()?
        };
        let mut mod_list = ModulesList::from_conf(paths.modlist_path())
            .with("load mod-list".to_string())
            .with(&ctx)
            .owe_data()?;
        mod_list.set_mods_local(paths.spec_path().clone());
        let workflow = SysWorkflows::load_from(paths.workflow_path())
            .with(&ctx)
            .owe(SysReason::Load.into())?;
        flag.mark_suc();
        Ok(Self {
            define,
            mod_list,
            local: Some(root.to_path_buf()),
            workflow,
        })
    }

    pub fn new(define: SysDefine, actions: SysWorkflows) -> Self {
        Self {
            define,
            mod_list: ModulesList::default(),
            local: None,
            workflow: actions,
        }
    }
}
#[async_trait]
impl RefUpdateable<()> for SysModelSpec {
    async fn update_local(
        &self,
        accessor: Accessor,
        _path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<()> {
        if let Some(local) = &self.local {
            let value = self.mod_list.update_local(accessor, local, options).await?;
            let path = local.join("vars.yml");
            if path.exists() {
                std::fs::remove_file(&path).owe_sys()?;
            }
            value.vars.save_yml(&path).owe_res()?;
            Ok(())
        } else {
            MainReason::from(ElementReason::Miss("local path".into())).err_result()
        }
    }
}

#[async_trait]
impl Localizable for SysModelSpec {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> MainResult<()> {
        if let Some(_local) = &self.local {
            self.mod_list.localize(dst_path, options).await?;
            Ok(())
        } else {
            MainReason::from(ElementReason::Miss("local path".into())).err_result()
        }
    }
}
impl SysModelSpec {
    pub fn for_example(name: &str) -> MainResult<SysModelSpec> {
        ModProject::make_test_prj("redis2_mock")?;
        ModProject::make_test_prj("mysql2_mock")?;
        make_sys_spec_test(
            SysDefine::new(name, ModelSTD::from_cur_sys()),
            vec!["redis2_mock", "mysql2_mock"],
        )
    }

    pub fn make_new(define: SysDefine) -> MainResult<SysModelSpec> {
        let actions = SysWorkflows::sys_tpl_init();
        let mut modul_spec = SysModelSpec::new(define.clone(), actions);
        let mod_name = "you_mod1";

        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                mod_name,
                GitRepository::from("https://github.com/you-mod1").with_tag("0.1.0"),
                ModelSTD::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
            )
            .with_enable(false)
            .with_setting(LocalizeVarPath::of_module(
                mod_name,
                define.model().to_string().as_str(),
            )),
        );
        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                "you_mod2",
                GitRepository::from("https://github.com/you-mod2").with_branch("beta"),
                ModelSTD::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
            )
            .with_enable(false),
        );
        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                "you_mod3",
                GitRepository::from("https://github.com/you-mod3").with_tag("v1.0.0"),
                ModelSTD::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
            )
            .with_enable(false),
        );
        Ok(modul_spec)
    }
}

pub fn make_sys_spec_test(define: SysDefine, mod_names: Vec<&str>) -> MainResult<SysModelSpec> {
    let actions = SysWorkflows::sys_tpl_init();
    let mut modul_spec = SysModelSpec::new(define, actions);
    for mod_name in mod_names {
        //let mod_name = "postgresql";
        let model = ModelSTD::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host);
        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                mod_name,
                LocalPath::from(format!("{MODULES_SPC_ROOT}/{mod_name}").as_str()),
                model.clone(),
            )
            .with_setting(LocalizeVarPath::of_module(
                mod_name,
                model.to_string().as_str(),
            )),
        );
    }

    Ok(modul_spec)
}

#[cfg(test)]
pub mod tests {

    use orion_error::TestAssertWithMsg;
    use orion_infra::path::make_clean_path;
    use orion_variate::tools::test_init;

    use crate::{
        accessor::accessor_for_test, const_vars::SYS_MODEL_SPC_ROOT, module::proj::ModProject,
    };

    use super::*;

    #[tokio::test]
    async fn build_example_sys_spec() -> MainResult<()> {
        test_init();
        let sys_name = "example_sys";
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT).join(sys_name);
        make_clean_path(&spec_root).owe_logic()?;
        ModProject::make_test_prj("redis_mock")?;
        ModProject::make_test_prj("mysql_mock")?;
        let spec = make_sys_spec_test(
            SysDefine::new(sys_name, ModelSTD::from_cur_sys()),
            vec!["redis_mock", "mysql_mock"],
        )
        .assert("make spec");
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT);
        let spec_path = spec_root.join(spec.define().name());
        make_clean_path(&spec_path).owe_logic()?;
        let accessor = accessor_for_test();
        spec.save_to(&spec_root).assert("spec save");
        let spec_path = spec_root.join(spec.define().name());
        let spec = SysModelSpec::load_from(&spec_path).assert("spec load");
        spec.update_local(accessor, &spec_path, &DownloadOptions::for_test())
            .await
            .assert("update");
        spec.localize(None, LocalizeOptions::for_test())
            .await
            .assert("localize");
        Ok(())
    }
}
