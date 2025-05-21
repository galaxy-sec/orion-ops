use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use crate::addr::AddrType;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OsType {
    MacOs,
    Ubuntu,
}
#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    cpe: String,
    meta: OsType,
    addr: AddrType,
}
impl Artifact {
    pub fn new<S: Into<String>, A: Into<AddrType>>(cep: S, meta: OsType, addr: A) -> Self {
        Self {
            meta,
            cpe: cep.into(),
            addr: addr.into(),
        }
    }
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct DockImage {
    cep: String,
    addr: AddrType,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct BinPackage {
    cep: String,
    addr: AddrType,
}
