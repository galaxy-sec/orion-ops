use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::From;
use serde_derive::{Deserialize, Serialize};

use crate::{
    error::SpecResult,
    types::{AsyncUpdateable, UpdateOptions},
    vars::EnvEvalable,
};

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
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        match self {
            AddrType::Git(addr) => addr.update_local(path, options).await,
            AddrType::Http(addr) => addr.update_local(path, options).await,
            AddrType::Local(addr) => addr.update_local(path, options).await,
        }
    }

    async fn update_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> SpecResult<PathBuf> {
        match self {
            AddrType::Git(addr) => addr.update_rename(path, name, options).await,
            AddrType::Http(addr) => addr.update_rename(path, name, options).await,
            AddrType::Local(addr) => addr.update_rename(path, name, options).await,
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

#[derive(Getters, Clone, Debug, Serialize, Deserialize, From, Default)]
#[serde(transparent)]
pub struct EnvVarPath {
    origin: String,
}
impl EnvVarPath {
    pub fn path(&self) -> PathBuf {
        let real = self.origin.clone().env_eval();
        PathBuf::from(real)
    }
}

impl From<&str> for EnvVarPath {
    fn from(value: &str) -> Self {
        Self {
            origin: value.to_string(),
        }
    }
}

impl From<PathBuf> for EnvVarPath {
    fn from(value: PathBuf) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

impl From<&PathBuf> for EnvVarPath {
    fn from(value: &PathBuf) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

impl From<&Path> for EnvVarPath {
    fn from(value: &Path) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}
