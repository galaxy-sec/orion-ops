use std::{fs, path::PathBuf};

use async_trait::async_trait;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde::{Serialize, de::DeserializeOwned};

use crate::{addr::rename_path, error::RunResult};

pub trait SaveAble<T> {
    fn save_to(&self, path: &PathBuf) -> RunResult<()>;
    fn load_from(path: &PathBuf) -> RunResult<T>;
}

#[async_trait]
pub trait AsyncUpdateable {
    async fn update_local(&self, path: &PathBuf) -> RunResult<PathBuf>;
    async fn update_rename(&self, path: &PathBuf, name: &str) -> RunResult<()> {
        let target = self.update_local(path).await?;
        rename_path(&target, name)?;
        Ok(())
    }
}

#[async_trait]
pub trait Localizeable {
    async fn localize(&self, path: &PathBuf) -> RunResult<()>;
}

pub trait TomlAble<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_toml(path: &PathBuf) -> RunResult<T>;
    fn save_toml(&self, path: &PathBuf) -> RunResult<()>;
}

impl<T> TomlAble<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_toml(path: &PathBuf) -> RunResult<T> {
        let mut ctx = WithContext::want("load object from toml");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        Ok(loaded)
    }
    fn save_toml(&self, path: &PathBuf) -> RunResult<()> {
        let mut ctx = WithContext::want("save toml");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = toml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}

pub trait IniAble<T>
where
    T: DeserializeOwned + Serialize,
{
    fn from_ini(path: &PathBuf) -> RunResult<T>;
    fn save_ini(&self, path: &PathBuf) -> RunResult<()>;
}

impl<T> IniAble<T> for T
where
    T: DeserializeOwned + Serialize,
{
    fn from_ini(path: &PathBuf) -> RunResult<T> {
        let mut ctx = WithContext::want("load object from toml");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_ini::de::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_ini(&self, path: &PathBuf) -> RunResult<()> {
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
    fn from_json(path: &PathBuf) -> RunResult<T>;
    fn save_json(&self, path: &PathBuf) -> RunResult<()>;
}

impl<T> JsonAble<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_json(path: &PathBuf) -> RunResult<T> {
        let mut ctx = WithContext::want("load object from toml");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_json::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_json(&self, path: &PathBuf) -> RunResult<()> {
        let mut ctx = WithContext::want("save toml");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = serde_json::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}
