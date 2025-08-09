use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use getset::Getters;
use orion_infra::path::{PathResult, ensure_path};
use orion_variate::{
    addr::accessor::UniversalAccessor,
    update::DownloadOptions,
    vars::{EnvDict, EnvEvalable, ValueDict, VarCollection},
};

use crate::error::MainResult;

pub type AnyResult<T> = anyhow::Result<T>;
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

pub type Accessor = Arc<UniversalAccessor>;
#[async_trait]
pub trait InsUpdateable<T> {
    //pub type UpdateObj = T;
    async fn update_local(
        self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<T>;
}

#[async_trait]
pub trait RefUpdateable<T> {
    //pub type UpdateObj = T;
    async fn update_local(
        &self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<T>;
}

#[derive(Clone, Debug, Default)]
pub struct LocalizeOptions {
    eval_dict: ValueDict,
    raw_dict: ValueDict,
    use_default_value: bool,
}
impl LocalizeOptions {
    pub fn new(raw_dict: ValueDict, mod_user_value: bool) -> Self {
        Self {
            eval_dict: raw_dict.clone().env_eval(&EnvDict::default()),
            raw_dict,
            use_default_value: mod_user_value,
        }
    }
    pub fn evaled_value(&self) -> &ValueDict {
        &self.eval_dict
    }
    pub fn raw_value(&self) -> &ValueDict {
        &self.raw_dict
    }
    pub fn use_default_value(&self) -> bool {
        self.use_default_value
    }

    pub fn for_test() -> Self {
        Self {
            eval_dict: ValueDict::new(),
            raw_dict: ValueDict::new(),
            use_default_value: false,
        }
    }
}

#[async_trait]
pub trait Localizable {
    async fn localize(
        &self,
        val_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> MainResult<()>;
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
