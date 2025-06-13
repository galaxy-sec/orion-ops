use std::{
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use derive_getters::Getters;
use derive_more::From;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    addr::rename_path,
    const_vars::{GLOBAL_VALUE_FILE, LOCAL_DIR},
    error::SpecResult,
};

pub trait Persistable<T> {
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()>;
    fn load_from(path: &Path) -> SpecResult<T>;
}

#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum UpdateLevel {
    #[default]
    All,
    Mod,
    Elm,
}

#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum RedoLevel {
    #[default]
    ReLocal,
    ReAll,
    ReChange,
}

impl From<usize> for RedoLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => RedoLevel::ReChange,
            1 => RedoLevel::ReLocal,
            2 => RedoLevel::ReAll,
            _ => RedoLevel::ReAll,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateOptions {
    re_level: RedoLevel,
    up_level: UpdateLevel,
}
impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            re_level: RedoLevel::default(),
            up_level: UpdateLevel::All,
        }
    }
}
impl UpdateOptions {
    pub fn new(re_level: RedoLevel, up_level: UpdateLevel) -> Self {
        Self { re_level, up_level }
    }
    pub fn redo_level(&self) -> RedoLevel {
        self.re_level.clone()
    }
    pub fn level(&self) -> UpdateLevel {
        self.up_level.clone()
    }
    pub fn for_test() -> Self {
        Self {
            re_level: RedoLevel::default(),
            up_level: UpdateLevel::All,
        }
    }
    pub fn for_depend() -> Self {
        Self {
            re_level: RedoLevel::ReChange,
            up_level: UpdateLevel::All,
        }
    }
}
#[async_trait]
pub trait AsyncUpdateable {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf>;
    async fn update_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> SpecResult<PathBuf> {
        let target = self.update_local(path, options).await?;
        rename_path(&target, name)
    }
}

#[derive(Clone, Debug, Getters)]
pub struct LocalizePath {
    local: PathBuf,
    value: PathBuf,
    global: Option<PathBuf>,
}
impl LocalizePath {
    pub fn new<P: AsRef<Path>>(local: P, value: P) -> Self {
        Self {
            local: PathBuf::from(local.as_ref()),
            value: PathBuf::from(value.as_ref()),
            global: None,
        }
    }
    pub fn from_root(root: &Path) -> Self {
        Self {
            local: root.join(LOCAL_DIR),
            value: root.join("value"),
            global: Some(root.join(GLOBAL_VALUE_FILE)),
        }
    }
    pub fn join_all<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            local: self.local.join(&path),
            value: self.value.join(&path),
            global: self.global.clone(),
        }
    }
    pub fn join<P: AsRef<Path>>(&self, local: P, value: P) -> Self {
        Self {
            local: self.local.join(&local),
            value: self.value.join(&value),
            global: self.global.clone(),
        }
    }
}
#[async_trait]
pub trait Localizable {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()>;
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
