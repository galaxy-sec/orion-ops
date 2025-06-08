use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use crate::{
    action::act::SysWorkflows,
    const_vars::{MOD_LIST_YML, NET_RES_YML, RESOURCE_YML, SPEC_DIR, VALUE_JSON, VARS_YML},
    error::ElementReason,
    types::{JsonAble, Localizable, LocalizePath},
    vars::{VarCollection, VarType},
};
use async_trait::async_trait;
use derive_getters::Getters;
use log::{error, info};
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};

use crate::{
    addr::GitAddr,
    error::{SpecReason, SpecResult, ToErr},
    module::{CpuArch, OsCPE, RunSPC, TargetNode, refs::ModuleSpecRef, spec::ModuleSpec},
    resource::{CaculateResSpec, Vps},
    task::{SetupTaskBuilder, TaskHandle},
    types::{Configable, Persistable},
};

use super::{
    ModelResource, ModulesList, NetAllocator, NetResSpace,
    init::{SysIniter, sys_init_gitignore},
};
#[derive(Getters, Clone, Debug)]
pub struct SysModelSpec {
    name: String,
    mod_list: ModulesList,
    vars: VarCollection,
    res: ModelResource,
    net: NetResSpace,
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
        let spec_path = target_root.join(SPEC_DIR);
        Self {
            target_root: target_root.to_path_buf(),
            net_path: spec_path.join(NET_RES_YML),
            res_path: spec_path.join(RESOURCE_YML),
            vars_path: target_root.join(VARS_YML),
            modlist_path: spec_path.join(MOD_LIST_YML),
            workflow_path: target_root.to_path_buf(),
            spec_path,
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

        let mut flag = log_flag!(
            info!(target: "spec/sys", "load sys spec success!:{}", root.display()),
            error!(target: "spec/sys", "load sys spec failed!:{}", root.display())
        );
        let paths = SysTargetPaths::from(&root);
        std::fs::create_dir_all(paths.spec_path()).owe_conf()?;
        sys_init_gitignore(&root)?;
        self.mod_list.save_conf(paths.modlist_path())?;

        self.res.save_conf(paths.res_path())?;
        self.net.save_conf(paths.net_path())?;
        self.vars.save_conf(paths.vars_path())?;
        self.workflow.save_to(paths.workflow_path(), None)?;
        flag.flag_suc();
        Ok(())
    }

    pub fn load_from(root: &Path) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load syspec");
        let name = root
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad name".to_string()))?;

        let mut flag = log_flag!(
            info!(target: "spec/sys", "load sys spec success!:{}", root.display()),
            error!(target: "spec/sys", "load sys spec failed!:{}", root.display())
        );
        let paths = SysTargetPaths::from(&root.to_path_buf());

        ctx.with_path("mod_list", paths.modlist_path());
        let mut mod_list = ModulesList::from_conf(paths.modlist_path())
            .with("load mod-list".to_string())
            .with(&ctx)?;
        mod_list.set_mods_local(paths.spec_path().clone());
        ctx.with_path("res_list", paths.res_path());
        let res = ModelResource::from_conf(paths.res_path()).with(&ctx)?;
        let net_res = NetResSpace::from_conf(paths.net_path()).with(&ctx)?;
        ctx.with_path("var_path", paths.vars_path());
        let vars = VarCollection::from_conf(paths.vars_path()).with(&ctx)?;
        let workflow = SysWorkflows::load_from(paths.workflow_path()).with(&ctx)?;
        flag.flag_suc();
        Ok(Self {
            name: name.to_string(),
            mod_list,
            vars,
            res,
            net: net_res,
            local: Some(root.to_path_buf()),
            workflow,
        })
    }

    pub fn new<S: Into<String>>(
        name: S,
        actions: SysWorkflows,
        net: NetResSpace,
        res: ModelResource,
        vars: VarCollection,
    ) -> Self {
        Self {
            name: name.into(),
            mod_list: ModulesList::default(),
            vars,
            res,
            net,
            local: None,
            workflow: actions,
        }
    }

    pub async fn update_local(&self) -> SpecResult<()> {
        if let Some(_local) = &self.local {
            self.mod_list.update().await?;
            Ok(())
        } else {
            SpecReason::from(ElementReason::Miss("local path".into())).err_result()
        }
    }
}

#[async_trait]
impl Localizable for SysModelSpec {
    async fn localize(&self, _dst_path: Option<LocalizePath>) -> SpecResult<()> {
        if let Some(local) = &self.local {
            let base_path = LocalizePath::from_root(local);
            let value_path = base_path.value().join(VALUE_JSON);
            if !value_path.exists() {
                value_path.parent().map(std::fs::create_dir_all);
                let export = self.vars().value_dict();
                export.save_json(&value_path)?
            }
            self.mod_list.localize(Some(base_path)).await?;
            Ok(())
        } else {
            SpecReason::from(ElementReason::Miss("local path".into())).err_result()
        }
    }
}
impl SetupTaskBuilder for SysModelSpec {
    fn make_setup_task(&self) -> SpecResult<TaskHandle> {
        self.mod_list().make_setup_task()
    }
}

pub fn make_sys_spec_example() -> SpecResult<SysModelSpec> {
    let repo = "https://e.coding.net/dy-sec/galaxy-open/modspec.git";
    let net = NetResSpace::new(
        Ipv4Addr::new(10, 0, 0, 1),
        (Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 10)),
    );

    let mut allocator = NetAllocator::new(net.clone());

    let res = ModelResource::from(vec![Vps::new(
        CaculateResSpec::new(4, 8),
        vec![allocator.alloc_master()],
    )]);
    let vars = VarCollection::define(vec![
        VarType::from(("IP", "10.0.0.1")),
        VarType::from(("pass", false)),
        VarType::from(("SPEED_LIMIT", 1000)),
    ]);

    let actions = SysWorkflows::sys_tpl_init();
    let mut modul_spec = SysModelSpec::new("example-sys", actions, net, res, vars);
    let mod_name = "example_mod1";
    modul_spec.add_mod_ref(
        ModuleSpecRef::from(
            mod_name,
            GitAddr::from(repo).branch("master").path(mod_name),
            TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        )
        .with_effective(false),
    );
    let mod_name = "postgresql";
    modul_spec.add_mod_ref(ModuleSpecRef::from(
        mod_name,
        GitAddr::from(repo).branch("master").path(mod_name),
        TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
    ));
    modul_spec.add_mod_ref(
        ModuleSpecRef::from(
            "mysql-example",
            GitAddr::from("http://github").tag("v1.0.0"),
            TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
        )
        .with_effective(false),
    );

    Ok(modul_spec)
}

pub fn make_sys_spec_new(name: &str, repo: &str) -> SpecResult<SysModelSpec> {
    let net = NetResSpace::new(
        Ipv4Addr::new(10, 0, 0, 1),
        (Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 10)),
    );

    let mut allocator = NetAllocator::new(net.clone());

    let res = ModelResource::from(vec![Vps::new(
        CaculateResSpec::new(4, 8),
        vec![allocator.alloc_master()],
    )]);
    let vars = VarCollection::define(vec![
        VarType::from(("IP", "10.0.0.1")),
        VarType::from(("pass", false)),
        VarType::from(("SPEED_LIMIT", 1000)),
    ]);

    let actions = SysWorkflows::sys_tpl_init();
    let mut modul_spec = SysModelSpec::new(name, actions, net, res, vars);
    modul_spec.add_mod_ref(
        ModuleSpecRef::from(
            "example_mod1",
            GitAddr::from(repo).path("example_mod1"),
            TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        )
        .with_effective(false),
    );
    modul_spec.add_mod_ref(
        ModuleSpecRef::from(
            "postgresql",
            GitAddr::from(repo).path("postgresql"),
            TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        )
        .with_effective(false),
    );
    modul_spec.add_mod_ref(
        ModuleSpecRef::from(
            "mysql-example",
            GitAddr::from("http://github").tag("v1.0.0"),
            TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
        )
        .with_effective(false),
    );

    Ok(modul_spec)
}

#[cfg(test)]
pub mod tests {

    use orion_error::TestAssertWithMsg;

    use crate::{
        addr::LocalAddr,
        const_vars::{MODULES_SPC_ROOT, SYS_MODEL_SPC_ROOT},
        tools::test_init,
    };

    use super::*;

    #[tokio::test]
    async fn build_example_sys_spec() -> SpecResult<()> {
        test_init();
        if PathBuf::from(SYS_MODEL_SPC_ROOT).exists() {
            std::fs::remove_dir_all(SYS_MODEL_SPC_ROOT).owe_res()?;
        }
        let spec = make_sys_spec_test().assert("make spec");
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT);
        spec.save_to(&spec_root).assert("spec save");
        let spec = SysModelSpec::load_from(&spec_root.join(spec.name())).assert("spec load");
        spec.update_local().await.assert("update");
        spec.localize(None).await.assert("localize");
        Ok(())
    }

    pub fn make_sys_spec_test() -> SpecResult<SysModelSpec> {
        let net = NetResSpace::new(
            Ipv4Addr::new(10, 0, 0, 1),
            (Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 10)),
        );

        let mut allocator = NetAllocator::new(net.clone());

        let res = ModelResource::from(vec![Vps::new(
            CaculateResSpec::new(4, 8),
            vec![allocator.alloc_master()],
        )]);
        let vars = VarCollection::define(vec![
            VarType::from(("IP", "10.0.0.1")),
            VarType::from(("pass", false)),
            VarType::from(("SPEED_LIMIT", 1000)),
        ]);

        let actions = SysWorkflows::sys_tpl_init();
        let mut modul_spec = SysModelSpec::new("example-sys", actions, net, res, vars);
        let mod_name = "example_mod1";
        modul_spec.add_mod_ref(
            ModuleSpecRef::from(
                mod_name,
                LocalAddr::from(format!("{}/{}", MODULES_SPC_ROOT, mod_name)),
                TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
            )
            .with_effective(false),
        );
        let mod_name = "postgresql";
        modul_spec.add_mod_ref(ModuleSpecRef::from(
            mod_name,
            LocalAddr::from(format!("{}/{}", MODULES_SPC_ROOT, mod_name)),
            TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        ));

        Ok(modul_spec)
    }
}
