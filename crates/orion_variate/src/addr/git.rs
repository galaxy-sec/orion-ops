use crate::types::RemoteUpdate;
use crate::vars::EnvEvalable;
use crate::{
    predule::*, tools::get_repo_name, types::LocalUpdate, update::UpdateOptions, vars::EnvDict,
};
use async_trait::async_trait;
use fs_extra::dir::CopyOptions;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    BranchType, FetchOptions, MergeOptions, PushOptions, RemoteUpdateFlags, Repository, ResetType,
};
use home::home_dir;
use log::warn;
use orion_error::UvsResFrom;
use orion_infra::auto_exit_log;
use orion_infra::path::ensure_path;

use super::AddrResult;

#[derive(Clone, Debug, Serialize, Deserialize, Default, Getters)]
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
impl EnvEvalable<GitAddr> for GitAddr {
    fn env_eval(self, dict: &EnvDict) -> GitAddr {
        Self {
            repo: self.repo.env_eval(dict),
            res: self.res.env_eval(dict),
            tag: self.tag.env_eval(dict),
            branch: self.branch.env_eval(dict),
            rev: self.rev.env_eval(dict),
            path: self.path.env_eval(dict),
            ssh_key: self.ssh_key.env_eval(dict),
            ssh_passphrase: self.ssh_passphrase.env_eval(dict),
        }
    }
}

impl GitAddr {
    pub fn from<S: Into<String>>(repo: S) -> Self {
        Self {
            repo: repo.into(),
            ..Default::default()
        }
    }
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = Some(tag.into());
        self
    }
    pub fn with_branch<S: Into<String>>(mut self, branch: S) -> Self {
        self.branch = Some(branch.into());
        self
    }
    pub fn with_rev<S: Into<String>>(mut self, rev: S) -> Self {
        self.rev = Some(rev.into());
        self
    }
    pub fn with_path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }
    // 新增：设置SSH私钥
    pub fn with_ssh_key<S: Into<String>>(mut self, ssh_key: S) -> Self {
        self.ssh_key = Some(ssh_key.into());
        self
    }
    // 新增：设置SSH密钥密码
    pub fn with_ssh_passphrase<S: Into<String>>(mut self, ssh_passphrase: S) -> Self {
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

    /// 更新现有仓库
    fn update_repo(&self, repo: &Repository) -> Result<(), git2::Error> {
        if !self.is_workdir_clean(repo)? {
            return Err(git2::Error::from_str("工作区有未提交的更改"));
        }
        // 1. 获取远程更新
        self.fetch_updates(repo)?;

        // 2. 处理检出目标（这会切换到指定分支）
        self.checkout_target(repo)?;

        // 3. 执行 pull 操作（合并远程变更）
        self.pull_updates(repo)
    }

    /// 执行 pull 操作：合并远程变更
    fn pull_updates(&self, repo: &Repository) -> Result<(), git2::Error> {
        // 获取当前分支信息
        let head = repo.head()?;
        let branch_name = match head.shorthand() {
            Some(name) => name,
            None => return Ok(()), // 分离头状态不需要 pull
        };

        // 获取上游分支信息
        let upstream_branch = format!("origin/{}", branch_name);
        let upstream_ref = match repo.find_reference(&upstream_branch) {
            Ok(r) => r,
            Err(_) => return Ok(()), // 没有上游分支
        };

        // 获取当前提交和上游提交
        let current_commit = head.peel_to_commit()?;
        let upstream_commit = upstream_ref.peel_to_commit()?;

        // 如果已经在最新状态，无需操作
        if current_commit.id() == upstream_commit.id() {
            return Ok(());
        }

        // 分析合并可能性
        let annotated_commit = repo.find_annotated_commit(upstream_commit.id())?;
        let analysis = repo.merge_analysis(&[&annotated_commit])?;
        //let analysis = repo.merge_analysis(&[&upstream_commit])?;

        if analysis.0.is_up_to_date() {
            // 已经是最新状态
            Ok(())
        } else if analysis.0.is_fast_forward() {
            // 执行快进合并
            self.fast_forward_merge(repo, &upstream_commit)
        } else {
            // 需要手动合并
            self.merge_upstream(repo, &upstream_commit)
        }
    }

    /// 执行快进合并
    fn fast_forward_merge(
        &self,
        repo: &Repository,
        upstream_commit: &git2::Commit,
    ) -> Result<(), git2::Error> {
        // 获取当前分支名称
        let refname = match repo.head()?.name() {
            Some(name) => name.to_string(),
            None => return Err(git2::Error::from_str("无法获取分支名称")),
        };

        // 更新引用到上游提交
        repo.reference(&refname, upstream_commit.id(), true, "Fast-forward")?;

        // 重置工作区到新提交
        repo.reset(upstream_commit.as_object(), ResetType::Hard, None)?;

        Ok(())
    }

    /// 执行非快进合并 (修复类型错误)
    fn merge_upstream(
        &self,
        repo: &Repository,
        upstream_commit: &git2::Commit,
    ) -> Result<(), git2::Error> {
        // 创建带注释的提交 (修复类型不匹配)
        let annotated_commit = repo.find_annotated_commit(upstream_commit.id())?;

        // 执行合并
        repo.merge(&[&annotated_commit], Some(&mut MergeOptions::new()), None)?;

        // 检查合并状态
        if repo.index()?.has_conflicts() {
            return Err(git2::Error::from_str("合并冲突：需要手动解决"));
        }

        // 创建合并提交
        let head_commit = repo.head()?.peel_to_commit()?;
        let mut index = repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;

        repo.commit(
            Some("HEAD"),
            &head_commit.author(),
            &head_commit.committer(),
            "合并远程变更",
            &tree,
            &[&head_commit, upstream_commit],
        )?;

        // 清理合并状态
        repo.cleanup_state()?;

        Ok(())
    }

    /// 检查工作区是否干净
    fn is_workdir_clean(&self, repo: &Repository) -> Result<bool, git2::Error> {
        let mut options = git2::StatusOptions::new();
        options.include_untracked(true);
        options.include_ignored(false);

        let statuses = repo.statuses(Some(&mut options))?;
        Ok(statuses.is_empty())
    }

    fn get_local_repo_name(&self) -> String {
        let mut name = get_repo_name(self.repo.as_str()).unwrap_or("unknow".into());
        if let Some(postfix) = self
            .rev
            .as_ref()
            .or(self.tag.as_ref())
            .or(self.branch.as_ref())
        {
            name = format!("{}_{}", name, postfix);
        }
        name
    }
}

#[async_trait]
impl LocalUpdate for GitAddr {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit> {
        let name = self.get_local_repo_name();
        let cache_local = home_dir()
            .ok_or(StructError::from_res("unget home".into()))?
            .join(".cache/galaxy");
        ensure_path(&cache_local).owe_logic()?;
        let mut git_local = cache_local.join(name.clone());
        let mut ctx = WithContext::want("update repository");

        ctx.with("repo", &self.repo);
        ctx.with_path("path", &git_local);
        let git_local_copy = git_local.clone();
        let mut flag = auto_exit_log!(
            info!(
                target : "addr/git",
                "update {} to {} success!", self.repo,git_local_copy.display()
            ),
            error!(
                target : "addr/git",
                "update {} to {} failed", self.repo,git_local_copy.display()
            )
        );
        debug!( target : "addr/git", "update options {:?} where :{} ", options, git_local.display() );
        if git_local.exists() && options.clean_git_cache() {
            std::fs::remove_dir_all(&git_local).owe_logic().with(&ctx)?;
            std::fs::create_dir_all(&git_local).owe_logic().with(&ctx)?;
            warn!(
                target : "addr/git",
                "remove cache {} from {} ", self.repo,git_local.display()
            )
        } else {
            debug!( target : "addr/git", "git_local:{} , clean : {} ",  git_local.exists(), options.clean_git_cache() );
        }

        match git2::Repository::open(&git_local) {
            Ok(re) => {
                debug!(target :"spec", "pull repo : {}", git_local.display());
                self.update_repo(&re).owe_data().with(&ctx)?;
            }
            Err(_) => {
                debug!(target :"spec", "clone repo : {}", git_local.display());
                self.clone_repo(&git_local).owe_data().with(&ctx)?;
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
        flag.mark_suc();
        Ok(UpdateUnit::from(real_path))
    }
}

#[async_trait]
impl RemoteUpdate for GitAddr {
    async fn update_remote(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit> {
        let ctx = WithContext::want("upload to repository");
        if !path.exists() {
            return Err(StructError::from_res("path not exist".into()));
        }
        let temp_path = home_dir().unwrap_or(PathBuf::from("~/")).join(".temp");
        ensure_path(&temp_path).owe_logic()?;

        let target_repo = self.update_local(&temp_path, options).await?;
        // 仓库地址
        let target_repo_in_local_path = &target_repo.position;
        if path.is_file() {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("UNKONW");
            std::fs::copy(path, target_repo_in_local_path.join(filename)).owe_res()?;
            std::fs::remove_file(path).owe_res()?;
        } else {
            let copy_options = CopyOptions::new().overwrite(true).copy_inside(true);
            fs_extra::copy_items(&[path], target_repo_in_local_path, &copy_options).owe_res()?;
            std::fs::remove_dir_all(path).owe_res()?;
        }
        match Repository::open(target_repo_in_local_path) {
            Ok(repo) => {
                let branch = self.branch.clone();
                self.submit(&repo, branch.unwrap_or("master".into()).as_str())
                    .owe_data()
                    .with(&ctx)?;
            }
            Err(e) => {
                debug!(target :"spec", "Open Local repo : {} is failed! error: {}", self.repo, e)
            }
        }
        let name = self.get_local_repo_name();
        std::fs::remove_dir_all(temp_path.join(name)).owe_res()?;
        Ok(UpdateUnit::from(path.to_path_buf()))
    }
}

impl GitAddr {
    pub fn sync_repo(&self, target_dir: &Path) -> Result<(), git2::Error> {
        // 尝试打开现有仓库
        match Repository::open(target_dir) {
            Ok(repo) => self.update_repo(&repo),
            Err(_) => self.clone_repo(target_dir),
        }
    }

    /// 克隆新仓库
    fn clone_repo(&self, target_dir: &Path) -> Result<(), git2::Error> {
        // 准备回调以支持认证
        //
        let callbacks = self.build_remote_callbacks(); // 使用构建的回调

        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 准备克隆选项
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // 执行克隆
        let repo = builder.clone(&self.repo, target_dir)?;

        // 处理检出目标
        self.checkout_target(&repo)
    }

    /// 获取远程更新
    fn fetch_updates(&self, repo: &Repository) -> Result<(), git2::Error> {
        // 查找 origin 远程
        let mut remote = repo.find_remote("origin")?;

        // 准备认证回调
        let callbacks = self.build_remote_callbacks(); // 使用构建的回调
                                                       // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 执行获取操作
        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;

        // 更新远程引用
        remote.update_tips(
            None,
            RemoteUpdateFlags::UPDATE_FETCHHEAD,
            git2::AutotagOption::All,
            None,
        )?;

        Ok(())
    }

    /// 处理检出目标（按优先级：rev > tag > branch）
    fn checkout_target(&self, repo: &Repository) -> Result<(), git2::Error> {
        if let Some(rev) = &self.rev {
            self.checkout_revision(repo, rev)
        } else if let Some(tag) = &self.tag {
            self.checkout_tag(repo, tag)
        } else if let Some(branch) = &self.branch {
            self.checkout_branch(repo, branch)
        } else {
            // 默认检出默认分支
            let head = repo.head()?;
            let _name = head
                .name()
                .ok_or_else(|| git2::Error::from_str("无法获取 HEAD 名称"))?;
            repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
            Ok(())
        }
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
                .ok_or_else(|| git2::Error::from_str("无效的分支名称"))?;
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

        Err(git2::Error::from_str(&format!("分支 '{}' 不存在", branch)))
    }

    /// 提交
    fn submit(&self, repo: &Repository, branch: &str) -> Result<(), git2::Error> {
        info!("git push origin {}", &branch);
        let branch_path = format!("refs/heads/{}", branch);
        // 拉取远程进行更新
        self.fetch_updates(repo)?;
        // 合并远程更新
        let fetch_head = repo.find_annotated_commit(repo.refname_to_id("FETCH_HEAD")?)?;
        let (analysis, _) = repo.merge_analysis(&[&fetch_head])?;

        if analysis.is_fast_forward() {
            let mut reference = repo.find_reference(&branch_path)?;
            reference.set_target(fetch_head.id(), "Fast-forward")?;
            repo.set_head(&branch_path)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else if analysis.is_normal() {
            // 出现合并冲突问题
            return Err(git2::Error::from_str("Merge conflicts detected"));
        }
        // 检查是否有修改
        let mut status_option = git2::StatusOptions::new();
        status_option.include_untracked(true);
        status_option.include_ignored(false);
        let local_repo_status = repo.statuses(Some(&mut status_option))?;
        if local_repo_status.is_empty() {
            return Ok(());
        }
        let mut origin = repo.find_remote("origin")?;

        // git add 操作
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree = repo.find_tree(index.write_tree()?)?;
        let head = repo.head()?.resolve()?;
        let parent_commit = repo.find_commit(head.target().unwrap())?;
        let signature = git2::Signature::now("dayu-spec", "dayu-sec-spec@dy-sec.com")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "spec:auto commit",
            &tree,
            &[&parent_commit],
        )?;
        // 设置认证信息
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(self.build_remote_callbacks());
        origin.push(
            &[&format!("{}:{}", branch_path, branch_path)],
            Some(&mut push_options),
        )?;
        info!("push complete");
        Ok(())
    }
}

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
    use crate::{addr::AddrResult, tools::test_init};

    use super::*;
    use orion_error::{ErrorOwe, TestAssert};
    use tempfile::tempdir;

    //git@e.coding.net:dy-sec/s-devkit/kubeconfig.git

    #[ignore = "need more time"]
    #[tokio::test]
    async fn test_git_addr_update_local() -> AddrResult<()> {
        // 创建临时目录

        let temp_dir = tempdir().owe_res()?;
        let dest_path = temp_dir.path().to_path_buf();

        // 使用一个小型测试仓库（这里使用 GitHub 上的一个测试仓库）
        let git_addr =
            GitAddr::from("https://github.com/galaxy-sec/hello-word.git").with_branch("master"); // 替换为实际测试分支

        // 执行克隆
        let cloned_v = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await?;

        // 验证克隆结果
        assert!(cloned_v.position().exists());
        assert!(cloned_v.position().join(".git").exists());

        // 验证分支/标签是否正确检出
        let repo = git2::Repository::open(cloned_v.position()).owe_res()?;
        let head = repo.head().owe_res()?;
        assert!(head.is_branch() || head.is_tag());

        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_update_local_sub() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./test/temp/git");
        //let target_path = dest_path.join("git");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();

        let git_addr = GitAddr::from("https://github.com/galaxy-sec/hello-word.git")
            .with_branch("main")
            .with_path("x86"); // 或使用 .tag("v1.0") 测试标签

        // 执行克隆
        let git_up = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await
            .assert();
        assert_eq!(git_up.position(), &dest_path.join("x86"));
        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_pull_2() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./test/temp/git2");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();

        let git_addr =
            GitAddr::from("https://github.com/galaxy-sec/hello-word.git").with_branch("main");
        // 执行克隆
        let git_up = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await
            .assert();
        assert_eq!(git_up.position(), &dest_path.join("hello-word.git_main"));
        Ok(())
    }

    #[tokio::test]
    async fn test_checkout_specific_branch() -> AddrResult<()> {
        test_init();
        let dest_path = PathBuf::from("./test/temp/git_branch_test");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).unwrap();
        }

        // 测试切换到非默认分支
        let git_addr =
            GitAddr::from("https://github.com/galaxy-sec/hello-word.git").with_branch("develop"); // 替换为实际测试分支

        let git_up = git_addr
            .update_local(&dest_path, &UpdateOptions::default())
            .await?;
        let repo = git2::Repository::open(git_up.position().clone()).assert();
        let head = repo.head().assert();
        assert!(head.shorthand().unwrap_or("").contains("develop"));
        Ok(())
    }

    use crate::types::RemoteUpdate;
    use crate::{addr::GitAddr, update::UpdateOptions};

    #[tokio::test]
    async fn test_dir_upload_to_remote_repo() -> AddrResult<()> {
        let temp_dir = tempdir().assert();
        let dir = temp_dir.path().join("version_1");
        let file = dir.join("test.txt");
        std::fs::create_dir_all(&dir).assert();
        std::fs::write(&file, "spec upload local dir to git repo.").assert();

        let git_addr = GitAddr::from("git@github.com:galaxy-sec/spec_test.git").with_branch("main");

        let git_up = git_addr
            .update_remote(&dir, &UpdateOptions::default())
            .await?;
        println!("{:?}", git_up.position);
        Ok(())
    }

    #[tokio::test]
    async fn test_file_upload_to_remote_repo() -> AddrResult<()> {
        let temp_dir = tempdir().assert();
        let file = temp_dir.path().join("test.txt");

        std::fs::write(&file, "spec upload local file to git repo.").assert();

        let git_addr = GitAddr::from("git@github.com:galaxy-sec/spec_test.git").with_branch("main");

        let git_up = git_addr
            .update_remote(&file, &UpdateOptions::default())
            .await?;
        println!("{:?}", git_up.position);
        Ok(())
    }
}
