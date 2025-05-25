use std::{
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde::{Serialize, de::DeserializeOwned};

use crate::{addr::rename_path, error::SpecResult};

pub trait Persistable<T> {
    fn save_to(&self, path: &Path) -> SpecResult<()>;
    fn load_from(path: &Path) -> SpecResult<T>;
}

#[async_trait]
pub trait AsyncUpdateable {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf>;
    async fn update_rename(&self, path: &Path, name: &str) -> SpecResult<()> {
        let target = self.update_local(path).await?;
        rename_path(&target, name)?;
        Ok(())
    }
}

#[async_trait]
pub trait Localizable {
    async fn localize(&self) -> SpecResult<()>;
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
        let mut ctx = WithContext::want("load object from toml");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        //let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        let loaded: T = serde_yaml::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_conf(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("save toml");
        ctx.with("path", format!("path: {}", path.display()));
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
        let mut ctx = WithContext::want("load object from toml");
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
        let mut ctx = WithContext::want("load object from toml");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_json::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_json(&self, path: &Path) -> SpecResult<()> {
        let mut ctx = WithContext::want("save toml");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = serde_json::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}
