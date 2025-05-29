use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};

use crate::{error::SpecResult, types::AsyncUpdateable};

use super::{GitAddr, HttpAddr, LocalAddr};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddrType {
    #[serde(rename = "git")]
    Git(GitAddr),
    #[serde(rename = "http")]
    Http(HttpAddr),
    #[serde(rename = "local")]
    Local(LocalAddr),
}

#[async_trait]
impl AsyncUpdateable for AddrType {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        match self {
            AddrType::Git(addr) => addr.update_local(path).await,
            AddrType::Http(addr) => addr.update_local(path).await,
            AddrType::Local(addr) => addr.update_local(path).await,
        }
    }

    async fn update_rename(&self, path: &Path, name: &str) -> SpecResult<()> {
        match self {
            AddrType::Git(addr) => addr.update_rename(path, name).await,
            AddrType::Http(addr) => addr.update_rename(path, name).await,
            AddrType::Local(addr) => addr.update_rename(path, name).await,
        }
    }
}

impl From<GitAddr> for AddrType {
    fn from(value: GitAddr) -> Self {
        Self::Git(value)
    }
}

impl From<HttpAddr> for AddrType {
    fn from(value: HttpAddr) -> Self {
        Self::Http(value)
    }
}

impl From<LocalAddr> for AddrType {
    fn from(value: LocalAddr) -> Self {
        Self::Local(value)
    }
}
