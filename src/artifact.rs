use derive_getters::Getters;
use derive_more::From;
use orion_variate::addr::AddrType;
use orion_variate::ext::Artifact;
use serde_derive::{Deserialize, Serialize};
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OsType {
    MacOs,
    Ubuntu,
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
