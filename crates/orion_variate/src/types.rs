use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    addr::{rename_path, AddrResult},
    update::UpdateOptions,
    vars::VarCollection,
};
use getset::{CloneGetters, CopyGetters, Getters, MutGetters, Setters, WithSetters};
use orion_infra::path::{ensure_path, PathResult};

#[derive(Clone, Getters, Setters, WithSetters, MutGetters, CopyGetters, CloneGetters, Default)]
pub struct UpdateUnit {
    #[getset(get = "pub", set = "pub", get_mut, set_with)]
    pub position: PathBuf,
    pub vars: Option<VarCollection>,
}
impl UpdateUnit {
    pub fn new(position: PathBuf, vars: VarCollection) -> Self {
        Self {
            position,
            vars: Some(vars),
        }
    }
    pub fn vars(&self) -> Option<&VarCollection> {
        self.vars.as_ref()
    }
}
impl From<PathBuf> for UpdateUnit {
    fn from(value: PathBuf) -> Self {
        Self {
            vars: None,
            position: value,
        }
    }
}

#[async_trait]
pub trait RemoteUpdate {
    async fn update_remote(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit>;
}
#[async_trait]
pub trait LocalUpdate {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit>;
    async fn update_local_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let mut target = self.update_local(path, options).await?;
        let path = rename_path(target.position(), name)?;
        target.set_position(path);
        Ok(target)
    }
}
