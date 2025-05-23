use std::{net::Ipv4Addr, path::PathBuf};

use derive_getters::Getters;
use orion_error::{ErrorOwe, StructError, UvsConfFrom};
use orion_exchange::vars::{ValueConstraint, VarCollection, VarType};

use crate::{
    addr::{GitAddr, LocalAddr},
    const_vars::MODULES_SPC_ROOT,
    error::SpecResult,
    module::{NodeType, refs::ModuleSpecRef, spec::ModuleSpec},
    resource::{CaculateResSpec, Vps},
    task::{SetupTaskBuilder, TaskHandle},
    types::{AsyncUpdateable, TomlAble},
};

use super::{ModelResource, ModulesList, NetAllocator, NetResSpace};
#[derive(Getters, Clone, Debug)]
pub struct SysModelSpec {
    name: String,
    mod_list: ModulesList,
    vars: VarCollection,
    res: ModelResource,
    net: NetResSpace,
}

impl SysModelSpec {
    pub fn add_mod(&mut self, modx: ModuleSpec) {
        self.mod_list.add_mod(modx);
    }
    pub fn add_mod_ref(&mut self, modx: ModuleSpecRef) {
        self.mod_list.add_ref(modx)
    }
    pub fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        self.save_local(path, &self.name())
    }
    pub fn save_local(&self, path: &PathBuf, name: &str) -> SpecResult<()> {
        let root = path.join(name);
        std::fs::create_dir_all(&root).owe_conf()?;
        let list_path = root.join("mod_list.toml");
        self.mod_list.save_toml(&list_path)?;

        let res_path = root.join("resource.toml");
        self.res.save_toml(&res_path)?;
        let net_path = root.join("net_res.toml");
        self.net.save_toml(&net_path)?;
        let var_path = root.join("vars.toml");
        self.vars.save_toml(&var_path)?;
        Ok(())
    }

    pub fn load_from(root: &PathBuf) -> SpecResult<Self> {
        let name = root
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad name".to_string()))?;

        let list_path = root.join("mod_list.toml");
        let mod_list = ModulesList::from_toml(&list_path)?;
        let res_path = root.join("resource.toml");
        let res = ModelResource::from_toml(&res_path)?;
        let net_path = root.join("net_res.toml");
        let net_res = NetResSpace::from_toml(&net_path)?;
        let var_path = root.join("vars.toml");
        let vars = VarCollection::from_toml(&var_path)?;
        Ok(Self {
            name: name.to_string(),
            mod_list,
            vars,
            res,
            net: net_res,
        })
    }

    pub fn new<S: Into<String>>(
        name: S,
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
        }
    }

    pub async fn assemble(&self, path: &PathBuf) -> SpecResult<()> {
        let root = path.join(self.name());
        self.mod_list.update_local(&root).await?;
        Ok(())
    }
}
impl SetupTaskBuilder for SysModelSpec {
    fn make_setup_task(&self) -> SpecResult<TaskHandle> {
        self.mod_list().make_setup_task()
    }
}

pub fn make_sys_spec_example() -> SpecResult<SysModelSpec> {
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

    let mut modul_spec = SysModelSpec::new("example-sys", net, res, vars);
    modul_spec.add_mod_ref(ModuleSpecRef::from(
        "example_mod1",
        LocalAddr::from(format!("{}/example_mod1", MODULES_SPC_ROOT)),
        NodeType::Host,
    ));
    modul_spec.add_mod_ref(ModuleSpecRef::from(
        "postgresql",
        LocalAddr::from(format!("{}/postgresql", MODULES_SPC_ROOT)),
        NodeType::Host,
    ));
    modul_spec.add_mod_ref(
        ModuleSpecRef::from(
            "mysql-example",
            GitAddr::from("http://github").tag("v1.0.0"),
            NodeType::K8s,
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
        let spec = make_sys_spec_example()?;
        let spec_root = PathBuf::from(SYS_MODEL_SPC_ROOT);
        spec.save_to(&spec_root)?;
        spec.assemble(&spec_root).await?;
        let _loaded = SysModelSpec::load_from(&spec_root.join(spec.name()))?;
        Ok(())
    }
}
