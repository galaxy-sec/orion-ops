use std::path::{Path, PathBuf};

use async_trait::async_trait;
use fs_extra::dir::CopyOptions;
use git2::{build::CheckoutBuilder, BranchType, FetchOptions, RemoteCallbacks, Repository};
use home::home_dir;
use log::{debug, error, info, warn};
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsResFrom, WithContext};
use serde_derive::{Deserialize, Serialize};

use crate::{
    error::SpecResult,
    log_guard,
    tools::get_repo_name,
    types::{AsyncUpdateable, UpdateOptions},
};

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
    // 新增：SSH私钥路径
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_key: Option<String>,
    // 新增：SSH密钥密码
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_passphrase: Option<String>,
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
    // 新增：设置SSH私钥
    pub fn ssh_key<S: Into<String>>(mut self, ssh_key: S) -> Self {
        self.ssh_key = Some(ssh_key.into());
        self
    }
    // 新增：设置SSH密钥密码
    pub fn ssh_passphrase<S: Into<String>>(mut self, ssh_passphrase: S) -> Self {
        self.ssh_passphrase = Some(ssh_passphrase.into());
        self
    }

    /// 构建远程回调（包含SSH认证）
    fn build_remote_callbacks(&self) -> git2::RemoteCallbacks<'_> {
        let mut callbacks = git2::RemoteCallbacks::new();
        let ssh_key = self.ssh_key.clone();
        let ssh_passphrase = self.ssh_passphrase.clone();

        callbacks.credentials(move |_url, username_from_url, allowed_types| {
            // 检查是否允许SSH认证
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                let username = username_from_url.unwrap_or("git");

                // 尝试获取SSH密钥路径
                let key_path = if let Some(custom_key) = &ssh_key {
                    // 使用用户指定的密钥
                    PathBuf::from(custom_key)
                } else {
                    // 自动查找常见默认密钥
                    find_default_ssh_key()
                        .ok_or_else(|| git2::Error::from_str("无法找到默认SSH密钥"))?
                };

                git2::Cred::ssh_key(
                    username,
                    None, // 不使用默认公钥路径
                    &key_path,
                    ssh_passphrase.as_deref(), // 传递密码（如果有）
                )
            } else {
                Err(git2::Error::from_str("不支持所需的认证类型"))
            }
        });
        callbacks
    }
}

#[async_trait]
impl AsyncUpdateable for GitAddr {
    async fn update_local(&self, path: &Path, _options: &UpdateOptions) -> SpecResult<PathBuf> {
        let name = get_repo_name(self.repo.as_str()).unwrap_or("unknow".into());
        let mut git_local = home_dir()
            .ok_or(StructError::from_res("unget home".into()))?
            .join(".galaxy/cache")
            .join(name.clone());
        let mut ctx = WithContext::want("update repository");

        ctx.with("repo", &self.repo);
        ctx.with_path("path", &git_local);
        let git_local_copy = git_local.clone();
        let mut flag = log_guard!(
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
            if let Some(sub_path) = PathBuf::from(sub).iter().next_back() {
                real_path = real_path.join(sub_path);
            }
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
        ctx.with("workflow", "pull code");
        let mut remote = repo.find_remote("origin").owe_res().with(&ctx)?;
        let cb = self.build_remote_callbacks(); // 使用构建的回调

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(cb); // 应用SSH回调
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
    /// 获取远程更新
    fn fetch_updates(&self, repo: &Repository) -> Result<(), git2::Error> {
        // 查找 origin 远程
        let mut remote = repo.find_remote("origin")?;

        // 准备认证回调
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, _allowed| {
            git2::Cred::userpass_plaintext(username.unwrap_or("git"), "")
        });

        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 执行获取操作
        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;

        // 更新远程引用
        remote.update_tips(None, true, git2::AutotagOption::All, None)?;

        Ok(())
    }
    /// 检出指定提交
    fn checkout_revision(&self, repo: &Repository, rev: &str) -> Result<(), git2::Error> {
        let obj = repo.revparse_single(rev)?;
        repo.checkout_tree(&obj, Some(&mut CheckoutBuilder::new().force()))?;
        repo.set_head_detached(obj.id())?;
        Ok(())
    }

    /// 检出指定标签
    fn checkout_tag(&self, repo: &Repository, tag: &str) -> Result<(), git2::Error> {
        let refname = format!("refs/tags/{}", tag);
        let obj = repo.revparse_single(&refname)?;
        repo.checkout_tree(&obj, Some(&mut CheckoutBuilder::new().force()))?;
        repo.set_head_detached(obj.id())?;
        Ok(())
    }

    /// 检出指定分支（包括远程分支）
    fn checkout_branch(&self, repo: &Repository, branch: &str) -> Result<(), git2::Error> {
        // 尝试查找本地分支
        if let Ok(b) = repo.find_branch(branch, BranchType::Local) {
            // 切换到本地分支
            let refname = b
                .get()
                .name()
                .ok_or(git2::Error::from_str("Invalid branch name"))?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
            return Ok(());
        }

        // 尝试查找远程分支
        let remote_branch_name = format!("origin/{}", branch);
        if let Ok(b) = repo.find_branch(&remote_branch_name, BranchType::Remote) {
            // 创建本地分支并设置跟踪
            let commit = b.get().peel_to_commit()?;
            let mut new_branch = repo.branch(branch, &commit, false)?;
            new_branch.set_upstream(Some(&format!("origin/{}", branch)))?;

            // 切换到新分支
            let refname = format!("refs/heads/{}", branch);
            repo.set_head(&refname)?;
            repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
            return Ok(());
        }

        Err(git2::Error::from_str(&format!(
            "Branch '{}' not found",
            branch
        )))
    }

    fn clone_repository(&self, path: &Path, mut ctx: WithContext) -> SpecResult<()> {
        ctx.with("workflow", "clone code");

        let mut fetch_options = git2::FetchOptions::new();
        let callbacks = self.build_remote_callbacks(); // 构建回调
        fetch_options.remote_callbacks(callbacks); // 应用SSH回调

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        let repo = builder.clone(&self.repo, path).owe_res().with(&ctx)?;

        // 处理检出目标（按优先级：rev > tag > branch）
        if let Some(rev) = &self.rev {
            self.checkout_revision(&repo, rev).owe_sys().with(&ctx)?;
        } else if let Some(tag) = &self.tag {
            self.checkout_tag(&repo, tag).owe_sys().with(&ctx)?;
        } else if let Some(branch) = &self.branch {
            self.checkout_branch(&repo, branch).owe_sys().with(&ctx)?;
        }

        Ok(())
    }
}

/// 查找常见的默认SSH密钥路径
fn find_default_ssh_key() -> Option<PathBuf> {
    // 获取用户主目录
    let home = home_dir()?;
    let ssh_dir = home.join(".ssh");

    // 尝试的密钥文件列表（按优先级排序）
    let key_files = [
        "id_ed25519", // 首选ed25519
        "id_rsa",     //  THEN 是RSA
        "id_ecdsa",   // 然后是ECDSA
        "identity",   // 通用名称
    ];

    // 检查每个密钥文件是否存在
    for key_file in &key_files {
        let key_path = ssh_dir.join(key_file);
        if key_path.exists() {
            return Some(key_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{error::SpecResult, tools::test_init};

    use super::*;
    use orion_error::{ErrorOwe, TestAssert};
    use tempfile::tempdir;

    //git@e.coding.net:dy-sec/s-devkit/kubeconfig.git

    #[ignore = "need more time"]
    #[tokio::test]
    async fn test_git_addr_update_local() -> SpecResult<()> {
        // 创建临时目录
        let temp_dir = tempdir().owe_res()?;
        let dest_path = temp_dir.path().to_path_buf();

        // 使用一个小型测试仓库（这里使用 GitHub 上的一个测试仓库）
        let git_addr =
            GitAddr::from("https://github.com/galaxy-sec/hello-word.git").branch("master"); // 替换为实际测试分支

        // 执行克隆
        let cloned_path = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await?;

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
            .path("postgresql/x86-ubt22-k8s"); // 或使用 .tag("v1.0") 测试标签

        // 执行克隆
        let real_path = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await
            .assert();
        assert_eq!(real_path, dest_path.join("x86-ubt22-k8s"));
        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_pull_2() -> SpecResult<()> {
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
        let real_path = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await
            .assert();
        assert_eq!(real_path, dest_path.join("modspec.git"));
        Ok(())
    }

    #[tokio::test]
    async fn test_checkout_specific_branch() -> SpecResult<()> {
        test_init();
        let dest_path = PathBuf::from("./test/temp/git_branch_test");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).unwrap();
        }

        // 测试切换到非默认分支
        let git_addr =
            GitAddr::from("https://github.com/galaxy-sec/hello-word.git").branch("develop"); // 替换为实际测试分支

        let real_path = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await?;
        let repo = git2::Repository::open(real_path).assert();
        let head = repo.head().assert();
        assert!(head.shorthand().unwrap_or("").contains("develop"));
        Ok(())
    }
}
