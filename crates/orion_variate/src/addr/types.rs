use crate::types::RemoteUpdate;
use crate::{predule::*, update::UpdateOptions, vars::EnvDict};
use derive_more::From;

use crate::{types::LocalUpdate, vars::EnvEvalable};

use super::{AddrResult, GitAddr, HttpAddr, LocalAddr};

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

impl EnvEvalable<AddrType> for AddrType {
    fn env_eval(self, dict: &EnvDict) -> AddrType {
        match self {
            AddrType::Git(v) => AddrType::Git(v.env_eval(dict)),
            AddrType::Http(v) => AddrType::Http(v.env_eval(dict)),
            AddrType::Local(v) => AddrType::Local(v.env_eval(dict)),
        }
    }
}

#[async_trait]
impl LocalUpdate for AddrType {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit> {
        let ins = self.clone().env_eval(options.values());
        match ins {
            AddrType::Git(addr) => addr.update_local(path, options).await,
            AddrType::Http(addr) => addr.update_local(path, options).await,
            AddrType::Local(addr) => addr.update_local(path, options).await,
        }
    }

    async fn update_local_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let ins = self.clone().env_eval(options.values());
        match ins {
            AddrType::Git(addr) => addr.update_local_rename(path, name, options).await,
            AddrType::Http(addr) => addr.update_local_rename(path, name, options).await,
            AddrType::Local(addr) => addr.update_local_rename(path, name, options).await,
        }
    }
}

#[async_trait]
impl RemoteUpdate for AddrType {
    async fn update_remote(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit> {
        let ins = self.clone().env_eval(options.values());
        match ins {
            AddrType::Git(addr) => addr.update_remote(path, options).await,
            AddrType::Http(addr) => addr.update_remote(path, options).await,
            AddrType::Local(addr) => addr.update_remote(path, options).await,
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
    pub fn path(&self, dict: &EnvDict) -> PathBuf {
        let real = self.origin.clone().env_eval(dict);
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
