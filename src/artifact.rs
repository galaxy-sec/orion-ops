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
//produce addr
//deploy addr
//translate addr
//release_source
//deploy_source
//transfrom_addr
#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    name: String,
    #[serde(alias = "addr")]
    deployment_repo: AddrType,
    transit_storage: Option<AddrType>,
    release_repo: Option<AddrType>,
    local: String,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize, From, Default)]
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
    pub fn new<S: Into<String>, A: Into<AddrType>>(name: S, addr: A, local: S) -> Self {
        Self {
            name: name.into(),
            deployment_repo: addr.into(),
            transit_storage: None,
            release_repo: None,
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
