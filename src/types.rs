use std::{
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    addr::rename_path,
    const_vars::VALUE_FILE,
    error::SpecResult,
    tools::ensure_path,
    update::UpdateOptions,
    vars::{ValueDict, VarCollection},
};
use getset::{CloneGetters, CopyGetters, Getters, MutGetters, Setters, WithSetters};

pub trait Persistable<T> {
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()>;
    fn load_from(path: &Path) -> SpecResult<T>;
}

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
pub trait UnitUpdateable {
    async fn update_local(
        &self,
        path: &Path,
        options: &UpdateOptions,
    ) -> SpecResult<UnitUpdateValue>;
    async fn update_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> SpecResult<UnitUpdateValue> {
        let mut target = self.update_local(path, options).await?;
        let path = rename_path(target.position(), name)?;
        target.set_position(path);
        Ok(target)
    }
}

#[async_trait]
pub trait SysUpdateable<T> {
    //pub type UpdateObj = T;
    async fn update_local(self, path: &Path, options: &UpdateOptions) -> SpecResult<T>;
}

#[derive(Clone, Debug, Getters)]
pub struct ValuePath {
    #[getset(get = "pub")]
    path: PathBuf,
}
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
    pub fn ensure_exist(self) -> SpecResult<Self> {
        ensure_path(&self.path)?;
        Ok(self)
    }
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

pub trait Configable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_conf(path: &Path) -> SpecResult<T>;
    fn save_conf(&self, path: &Path) -> SpecResult<()>;
}

impl<T> Configable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_conf(path: &Path) -> SpecResult<T> {
        T::from_yml(path)
    }
    fn save_conf(&self, path: &Path) -> SpecResult<()> {
        self.save_yml(path)
    }
}

pub trait IniAble<T>
where
    T: DeserializeOwned + Serialize,
{
    fn from_ini(path: &Path) -> SpecResult<T>;
    fn save_ini(&self, path: &Path) -> SpecResult<()>;
}

impl<T> IniAble<T> for T
where
    T: DeserializeOwned + Serialize,
{
    fn from_ini(path: &Path) -> SpecResult<T> {
        let mut ctx = WithContext::want("load object from ini");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_ini::de::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_ini(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("load conf spec");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = serde_ini::ser::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}

pub trait JsonAble<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_json(path: &Path) -> SpecResult<T>;
    fn save_json(&self, path: &Path) -> SpecResult<()>;
}

impl<T> JsonAble<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_json(path: &Path) -> SpecResult<T> {
        let mut ctx = WithContext::want("load object from json");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_json::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_json(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("save json");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = serde_json::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}

pub trait Tomlable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_toml(path: &Path) -> SpecResult<T>;
    fn save_toml(&self, path: &Path) -> SpecResult<()>;
}

impl<T> Tomlable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_toml(path: &Path) -> SpecResult<T> {
        let mut ctx = WithContext::want("load object from toml");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        Ok(loaded)
    }
    fn save_toml(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("save object to toml");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = toml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}

pub trait ValueConfable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_valconf(path: &Path) -> SpecResult<T>;
    fn save_valconf(&self, path: &Path) -> SpecResult<()>;
}

impl<T> ValueConfable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_valconf(path: &Path) -> SpecResult<T> {
        T::from_yml(path)
    }
    fn save_valconf(&self, path: &Path) -> SpecResult<()> {
        T::save_yml(self, path)
    }
}

pub trait Yamlable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_yml(path: &Path) -> SpecResult<T>;
    fn save_yml(&self, path: &Path) -> SpecResult<()>;
}

impl<T> Yamlable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_yml(path: &Path) -> SpecResult<T> {
        let mut ctx = WithContext::want("load object from yml");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        //let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        let loaded: T = serde_yaml::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_yml(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("save object fo yml");
        ctx.with_path("path", path);
        let data_content = serde_yaml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}
