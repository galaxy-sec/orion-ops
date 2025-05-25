use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use crate::{addr::AddrType, error::SpecResult, types::AsyncUpdateable};
use serde_derive::{Deserialize, Serialize};

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
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        self.addr.update_local(path).await
    }

    async fn update_rename(&self, path: &Path, name: &str) -> SpecResult<()> {
        self.addr.update_rename(path, name).await
    }
}
