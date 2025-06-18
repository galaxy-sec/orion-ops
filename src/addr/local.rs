use crate::{predule::*, update::ScopeLevel};

use contracts::debug_requires;
use fs_extra::dir::CopyOptions;

use crate::{log_guard, types::AsyncUpdateable, vars::EnvEvalable};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "local")]
pub struct LocalAddr {
    path: String,
}

impl EnvEvalable<LocalAddr> for LocalAddr {
    fn env_eval(self) -> LocalAddr {
        Self {
            path: self.path.env_eval(),
        }
    }
}
#[async_trait]
impl AsyncUpdateable for LocalAddr {
    //#[debug_ensures(matches!(*result, Ok(v) if v.exists()), "path not exists")]
    async fn update_local(&self, path: &Path, up_options: &UpdateOptions) -> SpecResult<PathBuf> {
        let mut ctx = WithContext::want("update local addr");
        ctx.with("src", self.path.as_str());
        ctx.with_path("dst", path);
        let src = PathBuf::from(self.path.as_str());
        let options = CopyOptions::new().overwrite(true); // 默认选项

        std::fs::create_dir_all(path).owe_res()?;
        let name = path_file_name(&src)?;
        let dst = path.join(name);
        let dst_copy = dst.clone();
        let mut flag = log_guard!(
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
        } else if dst.exists() && up_options.reuse_exists_path() {
            info!(
                target : "spec/addr/local",
                "ignore update {} to {} !", src.display(),dst_copy.display()
            );
        } else {
            fs_extra::dir::copy(&src, path, &options)
                .owe_data()
                .with(&ctx)?;
        }
        flag.flag_suc();
        Ok(dst)
    }

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
    let mut flag = log_guard!(
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
    use orion_error::TestAssert;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local() -> SpecResult<()> {
        let path = PathBuf::from("./test/temp/local");
        if path.exists() {
            std::fs::remove_dir_all(&path).owe_conf()?;
        }
        std::fs::create_dir_all(&path).owe_conf()?;
        let local = LocalAddr::from("./test/data/sys-1");
        local
            .update_rename(&path, "sys-2", &UpdateOptions::for_test())
            .await?;
        local
            .update_local(&path, &UpdateOptions::for_test())
            .await?;

        assert!(std::fs::exists(path.join("sys-2")).owe_conf()?);
        assert!(std::fs::exists(path.join("sys-1")).owe_conf()?);
        Ok(())
    }

    #[test]
    fn test_rename_path_file_new_target() -> SpecResult<()> {
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
    fn test_rename_path_file_overwrite_existing_file() -> SpecResult<()> {
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
    fn test_rename_path_dir_new_target() -> SpecResult<()> {
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
    fn test_rename_path_dir_overwrite_existing_dir() -> SpecResult<()> {
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
}
