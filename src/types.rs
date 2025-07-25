use std::path::{Path, PathBuf};

use async_trait::async_trait;
use getset::Getters;
use orion_infra::path::{PathResult, ensure_path};
use orion_variate::{
    update::UpdateOptions,
    vars::{ValueDict, VarCollection},
};

use crate::error::SpecResult;

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

#[async_trait]
pub trait SysUpdateable<T> {
    //pub type UpdateObj = T;
    async fn update_local(self, path: &Path, options: &UpdateOptions) -> SpecResult<T>;
}

#[derive(Clone, Debug, Default)]
pub struct LocalizeOptions {
    global_dict: ValueDict,
    use_default_value: bool,
}
impl LocalizeOptions {
    pub fn new(global_dict: ValueDict, mod_user_value: bool) -> Self {
        Self {
            global_dict,
            use_default_value: mod_user_value,
        }
    }
    pub fn global_value(&self) -> &ValueDict {
        &self.global_dict
    }
    pub fn with_global(mut self, value: ValueDict) -> Self {
        self.global_dict = value;
        self
    }
    pub fn use_default_value(&self) -> bool {
        self.use_default_value
    }

    pub fn for_test() -> Self {
        Self {
            global_dict: ValueDict::new(),
            use_default_value: false,
        }
    }
}

#[async_trait]
pub trait Localizable {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()>;
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
