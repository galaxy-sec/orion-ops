use std::{collections::HashMap, net::Ipv4Addr, path::PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, StructError, UvsConfFrom};
use serde_derive::{Deserialize, Serialize};

use crate::module::refs::ModuleSpecRef;
use crate::module::spec::ModuleSpec;
use crate::{
    addr::LocalAddr, const_vars::MODULES_SPC_ROOT, module::NodeType, resource::CaculateResSpec,
};
use crate::{
    addr::{AddrType, GitAddr},
    error::SpecResult,
    resource::{ResouceTypes, Vps},
    software::FileFormat,
    task::{CombinedTask, NodeSetupTaskBuilder, SetupTaskBuilder, TaskHandle},
    types::{AsyncUpdateable, TomlAble},
};

use orion_exchange::vars::{ValueConstraint, VarCollection, VarType};
#[derive(Getters, Clone, Debug)]
pub struct SysModelSpec {
    name: String,
    mod_list: ModulesList,
    vars: VarCollection,
    res: ModelResource,
    net: NetResSpace,
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysModelSpecRef {
    name: String,
    addr: AddrType,
}
impl SysModelSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(name: S, addr: A) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
        }
    }
}

#[async_trait]
impl AsyncUpdateable for SysModelSpecRef {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        self.addr.update_local(path).await
    }

    async fn update_rename(&self, path: &PathBuf, name: &str) -> SpecResult<()> {
        self.addr.update_rename(path, name).await
    }
}

#[derive(Getters, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModulesList {
    mods: Vec<ModuleSpecRef>,
    #[serde(skip)]
    mod_map: HashMap<String, ModuleSpec>,
}
impl ModulesList {
    pub fn add_ref(&mut self, spec_ref: ModuleSpecRef) {
        self.mods.push(spec_ref);
    }
}

#[async_trait]
impl AsyncUpdateable for ModulesList {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        let root = path.join("mods");
        std::fs::create_dir_all(&root).owe_data()?;
        for m in &self.mods {
            m.update_local(&root).await?;
        }
        Ok(root)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NoneValue<T> {
    None,
    Value(T),
}
impl ModulesList {
    pub fn add_mod(&mut self, modx: ModuleSpec) {
        self.mod_map.insert(modx.name().clone(), modx);
    }
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

impl SetupTaskBuilder for ModulesList {
    fn make_setup_task(&self) -> SpecResult<TaskHandle> {
        let mut task = CombinedTask::new("model setup");
        for item in &self.mods {
            if let Some(modx) = self.mod_map().get(item.name()) {
                task.add_sub(modx.make_setup_task(item.node())?);
            }
        }
        Ok(Box::new(task))
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    fmt: FileFormat,
    path: String,
}
impl ModelConfig {
    pub fn new<S: Into<String>>(fmt: FileFormat, path: S) -> Self {
        Self {
            fmt,
            path: path.into(),
        }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModelResource {
    res: Vec<ResouceTypes>,
}

impl From<Vec<Vps>> for ModelResource {
    fn from(value: Vec<Vps>) -> Self {
        let res = value.iter().map(ResouceTypes::from).collect();
        Self { res }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct NetResSpace {
    master: Ipv4Addr,
    node_scope: (Ipv4Addr, Ipv4Addr),
}
impl NetResSpace {
    pub fn new(master: Ipv4Addr, node_scope: (Ipv4Addr, Ipv4Addr)) -> Self {
        Self { master, node_scope }
    }
}
pub struct NetAllocator {
    net_res: NetResSpace,
    allocted: Vec<Ipv4Addr>,
}
impl NetAllocator {
    pub fn new(net_res: NetResSpace) -> Self {
        Self {
            net_res,
            allocted: Vec::new(),
        }
    }

    pub fn alloc_master(&mut self) -> Ipv4Addr {
        let master = self.net_res.master();
        self.allocted.push(*master);
        *master
    }

    pub fn alloc_node(&mut self) -> Option<Ipv4Addr> {
        let (start, end) = self.net_res.node_scope();
        for i in start.octets()[3]..=end.octets()[3] {
            let ip = Ipv4Addr::new(start.octets()[0], start.octets()[1], start.octets()[2], i);
            if !self.allocted.contains(&ip) {
                self.allocted.push(ip);
                return Some(ip);
            }
        }
        None
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
        "warpflow",
        //GitAddr::from("http://github").tag("v1.0.0"),
        LocalAddr::from(format!("{}/warpflow", MODULES_SPC_ROOT)),
        NodeType::Host,
    ));
    modul_spec.add_mod_ref(ModuleSpecRef::from(
        "mysql",
        LocalAddr::from(format!("{}/mysql", MODULES_SPC_ROOT)),
        NodeType::K8s,
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
