use crate::update::UpdateOptions;
use crate::{predule::*, types::RemoteUpdate, vars::EnvDict};
use contracts::debug_requires;
use fs_extra::dir::CopyOptions;
use orion_error::UvsResFrom;
use orion_infra::auto_exit_log;

use crate::{types::LocalUpdate, vars::EnvEvalable};

use super::AddrResult;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "local")]
pub struct LocalAddr {
    path: String,
}

impl EnvEvalable<LocalAddr> for LocalAddr {
    fn env_eval(self, dict: &EnvDict) -> LocalAddr {
        Self {
            path: self.path.env_eval(dict),
        }
    }
}
#[async_trait]
impl LocalUpdate for LocalAddr {
    //#[debug_ensures(matches!(*result, Ok(v) if v.exists()), "path not exists")]
    async fn update_local(
        &self,
        path: &Path,
        up_options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let mut ctx = WithContext::want("update local addr");
        ctx.with("src", self.path.as_str());
        ctx.with_path("dst", path);
        let src = PathBuf::from(self.path.as_str());
        let options = CopyOptions::new().overwrite(true); // 默认选项

        std::fs::create_dir_all(path).owe_res()?;
        let name = path_file_name(&src)?;
        let dst = path.join(name);
        let dst_copy = dst.clone();
        let mut flag = auto_exit_log!(
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
        } else if dst.exists() && up_options.copy_to_exists_path() {
            info!(
                target : "spec/addr/local",
                "ignore update {} to {} !", src.display(),dst_copy.display()
            );
        } else {
            fs_extra::dir::copy(&src, path, &options)
                .owe_data()
                .with(&ctx)?;
        }
        flag.mark_suc();
        Ok(UpdateUnit::from(dst))
    }

    async fn update_local_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let target = self.update_local(path, options).await?;
        Ok(UpdateUnit::from(rename_path(target.position(), name)?))
    }
}

#[async_trait]
impl RemoteUpdate for LocalAddr {
    async fn update_remote(&self, path: &Path, _: &UpdateOptions) -> AddrResult<UpdateUnit> {
        if !path.exists() {
            return Err(StructError::from_res("path not exist".into()));
        }
        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("UNKNOW");
            let target_path = Path::new(self.path()).join(file_name);
            std::fs::copy(path, target_path).owe_res()?;
            std::fs::remove_file(path).owe_res()?;
        } else {
            let copy_options = CopyOptions::new().overwrite(true).copy_inside(true);
            fs_extra::dir::copy(path, self.path(), &copy_options).owe_res()?;
            std::fs::remove_dir_all(path).owe_res()?;
        }
        Ok(UpdateUnit::from(path.to_path_buf()))
    }
}

pub fn path_file_name(path: &Path) -> AddrResult<String> {
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(StructError::from_conf("get file_name error".to_string()))?;
    Ok(file_name.to_string())
}
#[debug_requires(local.exists(), "local need exists")]
pub fn rename_path(local: &Path, name: &str) -> AddrResult<PathBuf> {
    let mut ctx = WithContext::want("rename path");
    let dst_path = local
        .parent()
        .map(|x| x.join(name))
        .ok_or(StructError::from_conf("bad path".to_string()))?;

    let dst_copy = dst_path.clone();
    let mut flag = auto_exit_log!(
        info!(target:"spec","rename {} to {} sucess!",local.display(),dst_copy.display()),
        error!(target:"spec","rename {} to {} failed!",local.display(),dst_copy.display())
    );
    if dst_path.exists() {
        if dst_path == local {
            flag.mark_suc();
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
    flag.mark_suc();
    Ok(dst_path)
}
impl LocalAddr {
    pub fn from<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }
}

#[cfg(test)]
mod tests {
    use crate::{addr::AddrResult, update::UpdateOptions};

    use super::*;
    use orion_error::TestAssert;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local() -> AddrResult<()> {
        let path = PathBuf::from("./test/temp/local");
        if path.exists() {
            std::fs::remove_dir_all(&path).owe_conf()?;
        }
        std::fs::create_dir_all(&path).owe_conf()?;
        let local = LocalAddr::from("./test/data/sys-1");
        local
            .update_local_rename(&path, "sys-2", &UpdateOptions::for_test())
            .await?;
        local
            .update_local(&path, &UpdateOptions::for_test())
            .await?;

        assert!(std::fs::exists(path.join("sys-2")).owe_conf()?);
        assert!(std::fs::exists(path.join("sys-1")).owe_conf()?);
        Ok(())
    }

    #[test]
    fn test_rename_path_file_new_model() -> AddrResult<()> {
        // 创建临时目录
        let temp_dir = tempdir().assert();
        let src_path = temp_dir.path().join("source.txt");
        std::fs::write(&src_path, "test content").assert();

        // 执行重命名（目标不存在）
        let renamed = rename_path(&src_path, "renamed.txt").assert();

        // 验证结果
        assert!(renamed.exists());
        assert!(!src_path.exists());
        assert_eq!(renamed.file_name().unwrap(), "renamed.txt");
        Ok(())
    }

    #[test]
    fn test_rename_path_file_overwrite_existing_file() -> AddrResult<()> {
        // 创建临时目录
        let temp_dir = tempdir().assert();
        let src_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("existing.txt");
        std::fs::write(&src_path, "source content").assert();
        std::fs::write(&target_path, "existing content").assert();

        // 执行重命名（覆盖现有文件）
        let renamed = rename_path(&src_path, "existing.txt").assert();

        // 验证结果
        assert!(renamed.exists());
        assert!(!src_path.exists());
        assert_eq!(std::fs::read_to_string(&renamed).assert(), "source content"); // 应覆盖原有内容
        Ok(())
    }

    #[test]
    fn test_rename_path_dir_new_model() -> AddrResult<()> {
        // 创建临时目录
        let temp_dir = PathBuf::from("./test/temp/rename_test");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).assert();
        }
        std::fs::create_dir_all(&temp_dir).assert();

        let src_dir = temp_dir.join("source_dir");
        let new_dir = temp_dir.join("renamed_dir");
        std::fs::create_dir(&src_dir).assert();
        std::fs::write(src_dir.join("file.txt"), "test").assert();

        // 执行重命名（目标不存在）
        let renamed = rename_path(&src_dir, "renamed_dir").assert();

        // 验证结果
        assert!(renamed.exists());
        assert!(renamed.join("file.txt").exists());
        assert!(!src_dir.exists());
        assert!(new_dir.exists());
        Ok(())
    }

    #[test]
    fn test_rename_path_dir_overwrite_existing_dir() -> AddrResult<()> {
        // 创建临时目录
        let temp_dir = tempdir().assert();
        let src_dir = temp_dir.path().join("source_dir");
        let target_dir = temp_dir.path().join("existing_dir");
        std::fs::create_dir(&src_dir).assert();
        std::fs::create_dir(&target_dir).assert();
        std::fs::write(src_dir.join("source_file.txt"), "source").assert();
        std::fs::write(target_dir.join("existing_file.txt"), "existing").assert();

        // 执行重命名（覆盖现有目录）
        let renamed = rename_path(&src_dir, "existing_dir")?;

        // 验证结果
        assert!(renamed.exists());
        assert!(renamed.join("source_file.txt").exists()); // 源目录内容应保留
        assert!(!renamed.join("existing_file.txt").exists()); // 原目标目录应被删除
        assert!(!src_dir.exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_file_from_local() -> AddrResult<()> {
        let target = tempdir().assert();
        let target_dir = target.path();
        let source = tempdir().assert();
        let source_dir = source.path();

        let file_path = source_dir.join("file.txt");
        std::fs::write(&file_path, "source").assert();
        let local_addr = LocalAddr::from(target_dir.to_str().unwrap_or("~/temp"));
        local_addr
            .update_remote(file_path.as_path(), &UpdateOptions::for_test())
            .await?;

        assert!(target_dir.join("file.txt").exists());
        assert!(!file_path.exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_dir_from_local() -> AddrResult<()> {
        let target = tempdir().assert();
        let target_dir = target.path();
        let source = tempdir().assert();
        let source_dir = source.path();

        let version_1 = target_dir.join("version_1");
        let version_2 = source_dir.join("version_2");
        std::fs::create_dir_all(&version_1).assert();
        std::fs::create_dir_all(&version_2).assert();

        std::fs::write(version_2.join("version_2.txt"), "version_2").assert();

        let local_addr = LocalAddr::from(version_1.to_str().unwrap_or("~/temp"));
        local_addr
            .update_remote(&version_2, &UpdateOptions::for_test())
            .await?;

        assert!(version_1.join("version_2").exists());
        assert!(version_1.join("version_2").join("version_2.txt").exists());
        assert!(!version_2.exists());
        Ok(())
    }
}
