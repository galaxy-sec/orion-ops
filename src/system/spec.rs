use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use crate::vars::{ValueConstraint, VarCollection, VarType};
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};

use crate::{
    action::act::Actions,
    addr::GitAddr,
    error::{SpecReason, SpecResult, ToErr},
    module::{CpuArch, OsCPE, RunSPC, TargetNode, refs::ModuleSpecRef, spec::ModuleSpec},
    resource::{CaculateResSpec, Vps},
    task::{SetupTaskBuilder, TaskHandle},
    types::{AsyncUpdateable, Configable, Persistable},
};

use super::{ModelResource, ModulesList, NetAllocator, NetResSpace, init::SysIniter};
#[derive(Getters, Clone, Debug)]
pub struct SysModelSpec {
    name: String,
    mod_list: ModulesList,
    vars: VarCollection,
    res: ModelResource,
    net: NetResSpace,
    local: Option<PathBuf>,
    actions: Actions,
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
        std::fs::create_dir_all(&root).owe_conf()?;
        let list_path = root.join(crate::const_vars::MOD_LIST_YML);
        self.mod_list.save_conf(&list_path)?;

        let res_path = root.join(crate::const_vars::RESOURCE_YML);
        self.res.save_conf(&res_path)?;
        let net_path = root.join(crate::const_vars::NET_RES_YML);
        self.net.save_conf(&net_path)?;
        let var_path = root.join(crate::const_vars::VARS_YML);
        self.vars.save_conf(&var_path)?;
        self.actions.save_to(&root)?;
        Ok(())
    }

    pub fn load_from(root: &Path) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load syspec");
        let name = root
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad name".to_string()))?;

        let list_path = root.join(crate::const_vars::MOD_LIST_YML);
        ctx.with_path("mod_list", &list_path);
        let mod_list = ModulesList::from_conf(&list_path).with(&ctx)?;
        let res_path = root.join(crate::const_vars::RESOURCE_YML);
        ctx.with_path("res_list", &res_path);
        let res = ModelResource::from_conf(&res_path).with(&ctx)?;
        let net_path = root.join(crate::const_vars::NET_RES_YML);
        let net_res = NetResSpace::from_conf(&net_path).with(&ctx)?;
        let var_path = root.join(crate::const_vars::VARS_YML);
        ctx.with_path("var_path", &var_path);
        let vars = VarCollection::from_conf(&var_path).with(&ctx)?;
        let actions = Actions::load_from(root).with(&ctx)?;
        Ok(Self {
            name: name.to_string(),
            mod_list,
            vars,
            res,
            net: net_res,
            local: Some(root.to_path_buf()),
            actions,
        })
    }

    pub fn new<S: Into<String>>(
        name: S,
        actions: Actions,
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
            actions,
        }
    }

    pub async fn update_local(&self) -> SpecResult<()> {
        if let Some(local) = &self.local {
            //let root = path.join(self.name());
            self.mod_list.update_local(local).await?;
            Ok(())
        } else {
            SpecReason::Miss("local path".into()).err_result()
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
        VarType::from(("SPEED_LIMIT", 1000)).constraint(ValueConstraint::scope(1000, 10000)),
    ]);

    let actions = Actions::sys_tpl_init();
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
        VarType::from(("SPEED_LIMIT", 1000)).constraint(ValueConstraint::scope(1000, 10000)),
    ]);

    let actions = Actions::sys_tpl_init();
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

    use crate::const_vars::SYS_MODEL_SPC_ROOT;

    use super::*;

    #[tokio::test]
    async fn build_example_sys_spec() -> SpecResult<()> {
        std::fs::remove_dir_all(SYS_MODEL_SPC_ROOT).owe_res()?;
        let spec = make_sys_spec_example()?;
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT);
        spec.save_to(&spec_root)?;
        let spec = SysModelSpec::load_from(&spec_root.join(spec.name()))?;
        spec.update_local().await?;
        Ok(())
    }
}
