pub mod init;
pub mod proj;
pub mod refs;
pub mod spec;
use crate::predule::*;
use std::{net::Ipv4Addr, path::PathBuf};

use crate::types::{Localizable, LocalizeOptions, SysUpdateValue, ValuePath};
use crate::vars::{ValueDict, ValueType, VarCollection};
use async_trait::async_trait;
use derive_more::Deref;

use crate::module::refs::ModuleSpecRef;
use crate::module::spec::ModuleSpec;
use crate::{
    error::SpecResult,
    resource::{ResouceTypes, Vps},
    software::FileFormat,
};

#[derive(Getters, Clone, Debug, Default, Serialize, Deserialize, Deref)]
#[serde(transparent)]
pub struct ModulesList {
    mods: Vec<ModuleSpecRef>,
    //#[serde(skip)]
    //mod_map: HashMap<String, ModuleSpec>,
}
impl ModulesList {
    pub fn add_ref(&mut self, spec_ref: ModuleSpecRef) {
        self.mods.push(spec_ref);
    }
    pub fn export(&self) -> ValueDict {
        let mut dict = ValueDict::new();
        for item in self.mods().iter() {
            if item.is_enable() {
                dict.insert(item.name(), ValueType::from(item.name().as_str()));
            }
        }
        dict
    }

    pub fn set_mods_local(&mut self, spec_path: PathBuf) {
        self.mods
            .iter_mut()
            .for_each(|x| x.set_local(spec_path.join("mods")));
    }

    pub fn find(&self, arg: &str) -> Option<&ModuleSpecRef> {
        self.mods.iter().find(|x| x.name() == arg)
    }
}

impl ModulesList {
    pub async fn update(
        &self,
        sys_root: &Path,
        options: &UpdateOptions,
    ) -> SpecResult<SysUpdateValue> {
        let mut vars = VarCollection::default();
        for m in &self.mods {
            if m.is_enable() {
                let update_v = m.update(sys_root, options).await?;
                if let Some(v) = update_v.vars {
                    vars = vars.merge(v);
                }
            }
        }
        Ok(SysUpdateValue::new(vars))
    }
    pub fn value_path(&self, parent: ValuePath) -> ValuePath {
        parent.join_all("mods")
    }
}
#[async_trait]
impl Localizable for ModulesList {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        let root = dst_path.map(|x| x.join_all("mods"));
        for m in &self.mods {
            if m.is_enable() {
                m.localize(root.clone(), options.clone()).await?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NoneValue<T> {
    None,
    Value(T),
}
impl ModulesList {
    pub fn add_mod(&mut self, _modx: ModuleSpec) {
        todo!();
        //self.mod_map.insert(modx.name().clone(), modx);
    }
}

/*
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
*/

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
