use std::path::PathBuf;

use async_trait::async_trait;
use derive_getters::Getters;
use fs_extra::dir::CopyOptions;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};
use serde_derive::{Deserialize, Serialize};

use crate::{error::SpecResult, types::AsyncUpdateable};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "local")]
pub struct LocalAddr {
    path: String,
}
#[async_trait]
impl AsyncUpdateable for LocalAddr {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        let mut ctx = WithContext::want("update local addr");
        ctx.with("src", self.path.as_str());
        ctx.with_path("dst", path);
        let src = PathBuf::from(self.path.as_str());
        let options = CopyOptions::new().overwrite(true); // 默认选项

        std::fs::create_dir_all(path).owe_res()?;
        let name = path_file_name(&src)?;
        let dst = path.join(name);
        if src.is_file() {
            std::fs::copy(&src, &dst).owe_res()?;
        } else {
            fs_extra::dir::copy(&src, path, &options)
                .owe_data()
                .with(&ctx)?;
        }
        Ok(dst)
    }

    async fn update_rename(&self, path: &PathBuf, name: &str) -> SpecResult<()> {
        let target = self.update_local(path).await?;
        rename_path(&target, name)?;
        Ok(())
    }
}

pub fn path_file_name(path: &PathBuf) -> SpecResult<String> {
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(StructError::from_conf("get file_name error".to_string()))?;
    Ok(file_name.to_string())
}
pub fn rename_path(local: &PathBuf, name: &str) -> SpecResult<()> {
    let mut ctx = WithContext::want("rename path");
    let new_src = local
        .parent()
        .map(|x| x.join(name))
        .ok_or(StructError::from_conf("bad path".to_string()))?;
    ctx.with("new-from", format!("{}", new_src.display()));
    std::fs::rename(local, &new_src).owe_conf().with(&ctx)?;
    Ok(())
}
impl LocalAddr {
    pub fn from<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local() -> SpecResult<()> {
        let path = PathBuf::from("./test/temp");
        std::fs::remove_dir_all(&path).owe_conf()?;
        std::fs::create_dir_all(&path).owe_conf()?;
        let local = LocalAddr::from("./test/data/sys-1");
        local.update_rename(&path, "sys-2").await?;
        local.update_local(&path).await?;

        assert!(std::fs::exists(path.join("sys-2")).owe_conf()?);
        assert!(std::fs::exists(path.join("sys-1")).owe_conf()?);
        Ok(())
    }
}
