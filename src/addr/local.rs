use std::path::{Path, PathBuf};

use async_trait::async_trait;
use contracts::debug_requires;
use derive_getters::Getters;
use fs_extra::dir::CopyOptions;
use log::{error, info};
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};
use serde_derive::{Deserialize, Serialize};

use crate::{error::SpecResult, log_flag, types::AsyncUpdateable};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "local")]
pub struct LocalAddr {
    path: String,
}
#[async_trait]
impl AsyncUpdateable for LocalAddr {
    //#[debug_ensures(matches!(*result, Ok(v) if v.exists()), "path not exists")]
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        let mut ctx = WithContext::want("update local addr");
        ctx.with("src", self.path.as_str());
        ctx.with_path("dst", path);
        let src = PathBuf::from(self.path.as_str());
        let options = CopyOptions::new().overwrite(true); // 默认选项

        std::fs::create_dir_all(path).owe_res()?;
        let name = path_file_name(&src)?;
        let dst = path.join(name);
        let dst_copy = dst.clone();
        let mut flag = log_flag!(
            info!(
                target : "spec/addr/local",
                "update {} to {} success!", src.display(),dst_copy.display()
            ),
            error!(
                target : "spec/addr/local",
                "update {} to {} failed", src.display(),dst_copy.display()
            )
        );

        if src.is_file() {
            std::fs::copy(&src, &dst).owe_res()?;
        } else {
            fs_extra::dir::copy(&src, path, &options)
                .owe_data()
                .with(&ctx)?;
        }
        flag.flag_suc();
        Ok(dst)
    }

    async fn update_rename(&self, path: &Path, name: &str) -> SpecResult<PathBuf> {
        let target = self.update_local(path).await?;
        rename_path(&target, name)
    }
}

pub fn path_file_name(path: &Path) -> SpecResult<String> {
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(StructError::from_conf("get file_name error".to_string()))?;
    Ok(file_name.to_string())
}
#[debug_requires(local.exists(), "local need exists")]
pub fn rename_path(local: &Path, name: &str) -> SpecResult<PathBuf> {
    let mut ctx = WithContext::want("rename path");
    let dst_path = local
        .parent()
        .map(|x| x.join(name))
        .ok_or(StructError::from_conf("bad path".to_string()))?;

    let dst_copy = dst_path.clone();
    let mut flag = log_flag!(
        info!(target:"spec","rename {} to {} sucess!",local.display(),dst_copy.display()),
        error!(target:"spec","rename {} to {} failed!",local.display(),dst_copy.display())
    );
    if dst_path.exists() {
        if dst_path == local {
            flag.flag_suc();
            return Ok(dst_path.clone());
        }
        if dst_path.is_dir() {
            std::fs::remove_dir_all(&dst_path)
                .owe_res()
                .with(&dst_path)
                .want("remove dst")?;
        } else {
            std::fs::remove_file(&dst_path)
                .owe_res()
                .with(&dst_path)
                .want("remove dst")?;
        }
    }
    ctx.with("new path", format!("{}", dst_path.display()));
    std::fs::rename(local, &dst_path).owe_conf().with(&ctx)?;
    flag.flag_suc();
    Ok(dst_path)
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
        let path = PathBuf::from("./test/temp/local");
        if path.exists() {
            std::fs::remove_dir_all(&path).owe_conf()?;
        }
        std::fs::create_dir_all(&path).owe_conf()?;
        let local = LocalAddr::from("./test/data/sys-1");
        local.update_rename(&path, "sys-2").await?;
        local.update_local(&path).await?;

        assert!(std::fs::exists(path.join("sys-2")).owe_conf()?);
        assert!(std::fs::exists(path.join("sys-1")).owe_conf()?);
        Ok(())
    }
}
