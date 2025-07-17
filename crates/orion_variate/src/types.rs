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
pub struct UnitUpdateValue {
    #[getset(get = "pub", set = "pub", get_mut, set_with)]
    pub position: PathBuf,
    pub vars: Option<VarCollection>,
}
impl UnitUpdateValue {
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
impl From<PathBuf> for UnitUpdateValue {
    fn from(value: PathBuf) -> Self {
        Self {
            vars: None,
            position: value,
        }
    }
}

#[derive(Clone)]
pub struct SysUpdateValue {
    pub vars: VarCollection,
}
impl SysUpdateValue {
    pub fn new(vars: VarCollection) -> Self {
        Self { vars }
    }
    pub fn vars(&self) -> &VarCollection {
        &self.vars
    }
}

#[async_trait]
pub trait ResourceUpload {
    async fn upload_from(
        &self,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UnitUpdateValue>;
}
#[async_trait]
pub trait UnitUpdateable {
    async fn update_local(
        &self,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UnitUpdateValue>;
    async fn update_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> AddrResult<UnitUpdateValue> {
        let mut target = self.update_local(path, options).await?;
        let path = rename_path(target.position(), name)?;
        target.set_position(path);
        Ok(target)
    }
}

#[derive(Clone, Debug, Getters)]
pub struct ValuePath {
    #[getset(get = "pub")]
    path: PathBuf,
}
pub const VALUE_FILE: &str = "value.yml";
impl ValuePath {
    pub fn new<P: AsRef<Path>>(value: P) -> Self {
        Self {
            //local: PathBuf::from(local.as_ref()),
            path: PathBuf::from(value.as_ref()),
        }
    }
    pub fn from_root(root: PathBuf) -> Self {
        Self { path: root }
    }
    pub fn join_all<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            //local: self.local.join(&path),
            path: self.path.join(&path),
        }
    }
    pub fn join<P: AsRef<Path>>(&self, value: P) -> Self {
        Self {
            //local: self.local.join(&local),
            path: self.path.join(&value),
        }
    }
    pub fn value_file(&self) -> PathBuf {
        self.path.join(VALUE_FILE)
    }
    pub fn ensure_exist(self) -> PathResult<Self> {
        ensure_path(&self.path)?;
        Ok(self)
    }
}
