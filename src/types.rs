use std::{
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    addr::rename_path,
    const_vars::{GLOBAL_JSON, LOCAL_DIR},
    error::SpecResult,
};

pub trait Persistable<T> {
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()>;
    fn load_from(path: &Path) -> SpecResult<T>;
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateOptions {
    force: bool,
}
impl Default for UpdateOptions {
    fn default() -> Self {
        Self { force: true }
    }
}
impl UpdateOptions {
    pub fn force(&self) -> bool {
        self.force
    }
    pub fn for_test() -> Self {
        Self { force: true }
    }
    pub fn for_depend() -> Self {
        Self { force: false }
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
            global: Some(root.join(GLOBAL_JSON)),
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
        let mut ctx = WithContext::want("load object from file");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        //let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        let loaded: T = serde_yaml::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_conf(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("save object fo file");
        ctx.with_path("path", path);
        let data_content = serde_yaml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
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
