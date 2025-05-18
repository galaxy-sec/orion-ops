use orion_error::ErrorOwe;
use orion_error::ErrorWith;
use orion_error::WithContext;
use serde_derive::Deserialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;

use derive_getters::Getters;
use serde_derive::Serialize;

use crate::error::RunResult;

#[derive(Debug, Clone)]
pub enum ResAddress {
    Ipv4(Ipv4Addr),
}

impl Display for ResAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResAddress::Ipv4(ipv4_addr) => {
                write!(f, "addr:{}", ipv4_addr)
            }
        }
    }
}
pub trait CaculateResource: Debug {
    fn address(&self) -> ResAddress;
}
pub type ResHold = Rc<dyn CaculateResource>;
pub type ResWeak = Weak<dyn CaculateResource>;
#[derive(Clone, Getters, Debug, Serialize, Deserialize)]
pub struct CaculateResSpec {
    core_cnt: u32,
    mem_size: u32,
}
impl CaculateResSpec {
    pub fn new(core_cnt: u32, mem_size: u32) -> Self {
        Self { core_cnt, mem_size }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResouceTypes {
    Vps(Vps),
}
impl ResouceTypes {
    pub fn address(&self) -> ResAddress {
        match self {
            ResouceTypes::Vps(vps) => vps.address(),
        }
    }
}
impl From<Vps> for ResouceTypes {
    fn from(value: Vps) -> Self {
        Self::Vps(value)
    }
}

impl From<&Vps> for ResouceTypes {
    fn from(value: &Vps) -> Self {
        Self::Vps(value.clone())
    }
}
#[derive(Getters, Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    name: String,
    items: Vec<ResouceTypes>,
}
pub type ResNodeRc = Rc<ResourceNode>;
impl ResourceNode {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            items: Vec::new(),
        }
    }
    pub fn add<R: Into<ResouceTypes>>(&mut self, res: R) {
        self.items.push(res.into())
    }
    pub fn try_load(path: &PathBuf) -> RunResult<Self> {
        let mut ctx = WithContext::want("load res node");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_conf().with(&ctx)?;

        let loaded: ResourceNode = toml::from_str(file_content.as_str())
            .owe_data()
            .with(&ctx)?;
        Ok(loaded)
    }
    pub fn save(&self, path: &PathBuf) -> RunResult<()> {
        let data_content = toml::to_string(self).owe_data()?;
        fs::write(path, data_content)
            .owe_conf()
            .with(format!("path: {}", path.display()))?;
        Ok(())
    }

    pub fn localhost(cpu: u32, mem: u32) -> Self {
        Self {
            name: "localhost".to_string(),
            items: vec![ResouceTypes::Vps(Vps::new(
                CaculateResSpec::new(cpu, mem),
                vec![],
            ))],
        }
    }
}

#[derive(Clone, Getters, Debug, Serialize, Deserialize)]
pub struct Vps {
    ips: Vec<Ipv4Addr>,
    res: CaculateResSpec,
}

impl Vps {
    pub fn new(res_spec: CaculateResSpec, mut ip: Vec<Ipv4Addr>) -> Self {
        let mut ip_list = vec![Ipv4Addr::new(127, 0, 0, 1)];
        ip_list.append(&mut ip);
        Self {
            res: res_spec,
            ips: ip_list,
        }
    }
}
impl CaculateResource for Vps {
    fn address(&self) -> ResAddress {
        ResAddress::Ipv4(self.ips.first().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{net::Ipv4Addr, path::PathBuf};

    #[test]
    fn test_resource_node_creation() {
        let node = ResourceNode::new("root");
        assert_eq!(node.name(), "root");
    }

    #[test]
    fn test_resource_node_save_load() {
        // 创建测试节点结构
        let mut root = ResourceNode::new("redis");
        let vps1 = Vps::new(
            CaculateResSpec::new(4, 16),
            vec![Ipv4Addr::new(10, 0, 0, 1)],
        );
        let vps2 = Vps::new(
            CaculateResSpec::new(4, 16),
            vec![Ipv4Addr::new(10, 0, 0, 2)],
        );
        let vps3 = Vps::new(
            CaculateResSpec::new(4, 16),
            vec![Ipv4Addr::new(10, 0, 0, 3)],
        );
        root.add(vps1);
        root.add(vps2);
        root.add(vps3);

        // 创建临时文件
        let temp_dir = PathBuf::from("./temp");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("node_redis.toml");

        root.save(&file_path).unwrap();

        let loaded = ResourceNode::try_load(&file_path).unwrap();
        // 验证文件内容

        // 验证数据完整性
        assert_eq!(root.name(), loaded.name());
    }
}
