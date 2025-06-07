use std::path::{Path, PathBuf};

use async_trait::async_trait;
use fs_extra::dir::CopyOptions;
use home::home_dir;
use log::{debug, error, info};
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsResFrom, WithContext};
use serde_derive::{Deserialize, Serialize};

use crate::{error::SpecResult, log_flag, tools::get_last_segment, types::AsyncUpdateable};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename = "git")]
pub struct GitAddr {
    repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    res: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

impl GitAddr {
    pub fn from<S: Into<String>>(repo: S) -> Self {
        Self {
            repo: repo.into(),
            ..Default::default()
        }
    }
    pub fn tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = Some(tag.into());
        self
    }
    pub fn branch<S: Into<String>>(mut self, branch: S) -> Self {
        self.branch = Some(branch.into());
        self
    }
    pub fn rev<S: Into<String>>(mut self, rev: S) -> Self {
        self.rev = Some(rev.into());
        self
    }
    pub fn path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }
}

#[async_trait]
impl AsyncUpdateable for GitAddr {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        let name = get_last_segment(self.repo.as_str()).unwrap_or("unknow".into());
        let mut git_local = home_dir()
            .ok_or(StructError::from_res("unget home".into()))?
            .join(".galaxy/cache")
            .join(name.clone());
        let mut ctx = WithContext::want("update repository");

        ctx.with("repo", &self.repo);
        ctx.with_path("path", &git_local);
        let git_local_copy = git_local.clone();
        let mut flag = log_flag!(
            info!(
                target : "spec/addr/git",
                "update {} to {} success!", self.repo,git_local_copy.display()
            ),
            error!(
                target : "spec/addr/local",
                "update {} to {} failed", self.repo,git_local_copy.display()
            )
        );

        match git2::Repository::open(&git_local) {
            Ok(re) => {
                debug!(target :"spec", "pull repo : {}", git_local.display());
                self.pull_repository(&re, ctx.clone())?;
            }
            Err(_) => {
                debug!(target :"spec", "clone repo : {}", git_local.display());
                self.clone_repository(&git_local, ctx.clone())?;
            }
        }
        let mut real_path = path.to_path_buf();
        if let Some(sub) = &self.path {
            git_local = git_local.join(sub);
            real_path = real_path.join(sub);
        } else {
            real_path = real_path.join(name);
        }
        if real_path.exists() {
            std::fs::remove_dir_all(&real_path).owe_res().with(&ctx)?;
        }

        std::fs::create_dir_all(&real_path).owe_res().with(&ctx)?;
        let options = CopyOptions::new();
        debug!(target:"spec", "src-path:{}", git_local.display() );
        debug!(target:"spec", "dst-path:{}", path.display() );
        ctx.with_path("src-path", &git_local);
        ctx.with_path("dst-path", &real_path);
        fs_extra::copy_items(&[&git_local], path, &options)
            .owe_res()
            .with(&ctx)?;
        flag.flag_suc();
        Ok(real_path)
    }
}

impl GitAddr {
    fn pull_repository(&self, repo: &git2::Repository, mut ctx: WithContext) -> SpecResult<()> {
        ctx.with("action", "pull code");
        let mut remote = repo.find_remote("origin").owe_res().with(&ctx)?;

        let mut fetch_options = git2::FetchOptions::new();
        let refspecs: &[&str] = &[]; //
        remote
            .fetch(refspecs, Some(&mut fetch_options), None)
            .owe_res()
            .with(&ctx)?;
        let fetch_head = repo.find_reference("FETCH_HEAD").owe_res().with(&ctx)?;
        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .owe_res()
            .with(&ctx)?;
        let analysis = repo.merge_analysis(&[&fetch_commit]).owe_res().with(&ctx)?;

        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.branch.as_deref().unwrap_or("master"));
            let mut reference = repo.find_reference(&refname).owe_res().with(&ctx)?;
            reference
                .set_target(fetch_commit.id(), "Fast-forward")
                .owe_res()
                .with(&ctx)?;
            repo.set_head(&refname).owe_res().with(&ctx)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .owe_res()
                .with(&ctx)?;
            Ok(())
        } else {
            Err(StructError::from_res("需要手动合并变更".into()).with(&ctx))
        }
    }

    fn clone_repository(&self, path: &PathBuf, mut ctx: WithContext) -> SpecResult<()> {
        ctx.with("action", "clone code");
        let repo = git2::Repository::clone(&self.repo, path)
            .owe_res()
            .with(&ctx)?;

        // 处理分支/标签
        if let Some(branch_or_tag) = self.branch.as_ref().or(self.tag.as_ref()) {
            let (_object, reference) = repo.revparse_ext(branch_or_tag).owe_res().with(&ctx)?;

            if let Some(reference) = reference {
                repo.set_head(reference.name().unwrap())
                    .owe_res()
                    .with(&ctx)?;
            }
        }

        // 处理特定 revision
        if let Some(rev) = &self.rev {
            let obj = repo.revparse_single(rev).owe_res().with(&ctx)?;
            repo.checkout_tree(&obj, None).owe_res().with(&ctx)?;
            repo.set_head_detached(obj.id()).owe_res().with(&ctx)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::SpecResult, tools::test_init};

    use super::*;
    use orion_error::{ErrorOwe, TestAssert};
    use tempfile::tempdir;

    #[ignore = "need more time"]
    #[tokio::test]
    async fn test_git_addr_update_local() -> SpecResult<()> {
        // 创建临时目录
        let temp_dir = tempdir().owe_res()?;
        let dest_path = temp_dir.path().to_path_buf();

        // 使用一个小型测试仓库（这里使用 GitHub 上的一个测试仓库）
        let git_addr = GitAddr::from("https://github.com/octocat/Hello-World.git").branch("master"); // 或使用 .tag("v1.0") 测试标签

        // 执行克隆
        let cloned_path = git_addr.update_local(&dest_path).await?;

        // 验证克隆结果
        assert!(cloned_path.exists());
        assert!(cloned_path.join(".git").exists());

        // 验证分支/标签是否正确检出
        let repo = git2::Repository::open(&cloned_path).owe_res()?;
        let head = repo.head().owe_res()?;
        assert!(head.is_branch() || head.is_tag());

        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_update_local_sub() -> SpecResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./test/temp/git");
        //let target_path = dest_path.join("git");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();

        let git_addr = GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/modspec.git")
            .branch("master")
            .path("postgresql"); // 或使用 .tag("v1.0") 测试标签

        // 执行克隆
        let real_path = git_addr.update_local(&dest_path).await.assert();
        assert_eq!(real_path, dest_path.join("postgresql"));
        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_pull_hole() -> SpecResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./test/temp/git2");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();

        let git_addr =
            GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/modspec.git").branch("master");
        //.path("*");
        //;

        // 执行克隆
        let real_path = git_addr.update_local(&dest_path).await.assert();
        assert_eq!(real_path, dest_path.join("modspec.git"));
        Ok(())
    }
}
