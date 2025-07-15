pub use derive_getters::Getters;
use derive_more::From;
use orion_error::{ErrorCode, UvsReason};
pub use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};
use serde::de::DeserializeOwned;
pub use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum SerdeReason {
    #[error("brief {0}")]
    Brief(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for SerdeReason {
    fn error_code(&self) -> i32 {
        match self {
            SerdeReason::Brief(_) => 500,
            SerdeReason::Uvs(r) => r.error_code(),
        }
    }
}

pub type SerdeResult<T> = Result<T, StructError<SerdeReason>>;
pub type SerdeError = StructError<SerdeReason>;
pub trait Persistable<T> {
    fn save_to(&self, path: &Path, name: Option<String>) -> SerdeResult<()>;
    fn load_from(path: &Path) -> SerdeResult<T>;
}

pub trait Configable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_conf(path: &Path) -> SerdeResult<T>;
    fn save_conf(&self, path: &Path) -> SerdeResult<()>;
}

impl<T> Configable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_conf(path: &Path) -> SerdeResult<T> {
        T::from_yml(path)
    }
    fn save_conf(&self, path: &Path) -> SerdeResult<()> {
        self.save_yml(path)
    }
}

pub trait IniAble<T>
where
    T: DeserializeOwned + serde::Serialize,
{
    fn from_ini(path: &Path) -> SerdeResult<T>;
    fn save_ini(&self, path: &Path) -> SerdeResult<()>;
}

impl<T> IniAble<T> for T
where
    T: DeserializeOwned + serde::Serialize,
{
    fn from_ini(path: &Path) -> SerdeResult<T> {
        let mut ctx = WithContext::want("load object from ini");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_ini::de::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_ini(&self, path: &Path) -> SerdeResult<()> {
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
    fn from_json(path: &Path) -> SerdeResult<T>;
    fn save_json(&self, path: &Path) -> SerdeResult<()>;
}

impl<T> JsonAble<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_json(path: &Path) -> SerdeResult<T> {
        let mut ctx = WithContext::want("load object from json");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = serde_json::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_json(&self, path: &Path) -> SerdeResult<()> {
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
    fn from_toml(path: &Path) -> SerdeResult<T>;
    fn save_toml(&self, path: &Path) -> SerdeResult<()>;
}

impl<T> Tomlable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_toml(path: &Path) -> SerdeResult<T> {
        let mut ctx = WithContext::want("load object from toml");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        Ok(loaded)
    }
    fn save_toml(&self, path: &Path) -> SerdeResult<()> {
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
    fn from_valconf(path: &Path) -> SerdeResult<T>;
    fn save_valconf(&self, path: &Path) -> SerdeResult<()>;
}

impl<T> ValueConfable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_valconf(path: &Path) -> SerdeResult<T> {
        T::from_yml(path)
    }
    fn save_valconf(&self, path: &Path) -> SerdeResult<()> {
        T::save_yml(self, path)
    }
}

pub trait Yamlable<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_yml(path: &Path) -> SerdeResult<T>;
    fn save_yml(&self, path: &Path) -> SerdeResult<()>;
}

impl<T> Yamlable<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn from_yml(path: &Path) -> SerdeResult<T> {
        let mut ctx = WithContext::want("load object from yml");
        ctx.with_path("path", path);
        let file_content = fs::read_to_string(path).owe_res().with(&ctx)?;
        //let loaded: T = toml::from_str(file_content.as_str()).owe_res().with(&ctx)?;
        let loaded: T = serde_yaml::from_str(file_content.as_str())
            .owe_res()
            .with(&ctx)?;
        Ok(loaded)
    }
    fn save_yml(&self, path: &Path) -> SerdeResult<()> {
        let mut ctx = WithContext::want("save object fo yml");
        ctx.with_path("path", path);
        let data_content = serde_yaml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }
}
