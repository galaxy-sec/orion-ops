use derive_more::{Deref, DerefMut};
use getset::Getters;
use orion_variate::addr::Address;
use serde_derive::{Deserialize, Serialize};

use crate::system::spec::SysDefine;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[getset(get = "pub")]
pub struct OpsSystem {
    sys: SysDefine,
    addr: Address,
}

impl OpsSystem {
    pub fn new(sys: SysDefine, addr: Address) -> Self {
        Self { sys, addr }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Default, Deref, DerefMut)]
pub struct OpsTarget {
    sys_models: Vec<OpsSystem>,
}
