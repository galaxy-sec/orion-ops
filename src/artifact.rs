use derive_getters::Getters;
use derive_more::From;
use serde_derive::{Deserialize, Serialize};
use std::ops::Deref;
use std::ops::DerefMut;

use crate::addr::AddrType;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OsType {
    MacOs,
    Ubuntu,
}
#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    name: String,
    addr: AddrType,
    local: String,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize, From)]
#[serde(transparent)]
pub struct ArtifactPackage {
    items: Vec<Artifact>,
}
impl Deref for ArtifactPackage {
    type Target = Vec<Artifact>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
impl DerefMut for ArtifactPackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl Artifact {
    pub fn new<S: Into<String>, A: Into<AddrType>>(
        name: S,
        addr: A,
        local: S,
    ) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
            local: local.into(),
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
