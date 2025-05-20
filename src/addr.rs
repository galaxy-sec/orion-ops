use std::path::PathBuf;

use async_trait::async_trait;
use derive_getters::Getters;
use fs_extra::dir::CopyOptions;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, UvsResFrom, WithContext};
use serde_derive::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::{error::SpecResult, types::AsyncUpdateable};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename = "git")]
pub struct GitAddr {
    repo: String,
    res: Option<String>,
    tag: Option<String>,
    branch: Option<String>,
    rev: Option<String>,
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
}

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

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "http")]
pub struct HttpAddr {
    url: String,
    username: Option<String>,
    password: Option<String>,
}

impl HttpAddr {
    pub fn from<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            username: None,
            password: None,
        }
    }

    pub fn with_credentials<S: Into<String>>(mut self, username: S, password: S) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }
}
#[async_trait]
impl AsyncUpdateable for HttpAddr {
    async fn update_local(&self, dest_dir: &PathBuf) -> SpecResult<PathBuf> {
        let client = reqwest::Client::new();

        // 构建请求
        let mut request = client.get(&self.url);

        // 添加Basic Auth
        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            request = request.basic_auth(u, Some(p));
        }

        // 发送请求（-fL 等效于 follow redirects + fail on error）
        let response = request
            .send()
            .await
            .owe_res()
            .with(format!("Failed to download {}", self.url))?;

        // 检查状态码（-f 选项行为）
        if !response.status().is_success() {
            return Err(StructError::from_res(format!(
                "HTTP request failed: {}",
                response.status()
            )));
        }

        // 获取文件名（从URL或Content-Disposition头）
        let file_name = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("downloaded_file");

        // 创建目标文件（-o 选项行为）
        let dest_path = dest_dir.join(file_name);
        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .owe_conf()
            .with(format!("Failed to create {}", dest_path.display()))?;

        // 流式写入文件
        let stream = response.bytes().await.owe_data()?;
        file.write_all(&stream)
            .await
            .owe_sys()
            .with(format!("Failed to write to {}", dest_path.display()))?;

        Ok(dest_path)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AddrType {
    #[serde(rename = "git")]
    Git(GitAddr),
    #[serde(rename = "http")]
    Http(HttpAddr),
    #[serde(rename = "local")]
    Local(LocalAddr),
}
impl GitAddr {}

#[async_trait]
impl AsyncUpdateable for GitAddr {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        let mut ctx = WithContext::want("git clone repository");
        ctx.with("repo", &self.repo);
        ctx.with("dest", format!("{}", path.display()));

        let repo = match git2::Repository::clone(&self.repo, path) {
            Ok(repo) => repo,
            Err(e) => return Err(StructError::from_res(format!("Git clone failed: {}", e))),
        };

        // 处理分支/标签
        if let Some(branch_or_tag) = self.branch.as_ref().or(self.tag.as_ref()) {
            let (object, reference) = repo.revparse_ext(branch_or_tag).owe_res().with(&ctx)?;

            repo.checkout_tree(&object, None).owe_res().with(&ctx)?;

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

        Ok(path.clone())
    }
}

#[async_trait]
impl AsyncUpdateable for AddrType {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        match self {
            AddrType::Git(addr) => addr.update_local(path).await,
            AddrType::Http(addr) => addr.update_local(path).await,
            AddrType::Local(addr) => addr.update_local(path).await,
        }
    }

    async fn update_rename(&self, path: &PathBuf, name: &str) -> SpecResult<()> {
        match self {
            AddrType::Git(addr) => addr.update_rename(path, name).await,
            AddrType::Http(addr) => addr.update_rename(path, name).await,
            AddrType::Local(addr) => addr.update_rename(path, name).await,
        }
    }
}

impl From<GitAddr> for AddrType {
    fn from(value: GitAddr) -> Self {
        Self::Git(value)
    }
}

impl From<HttpAddr> for AddrType {
    fn from(value: HttpAddr) -> Self {
        Self::Http(value)
    }
}

impl From<LocalAddr> for AddrType {
    fn from(value: LocalAddr) -> Self {
        Self::Local(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};
    use tempfile::tempdir;

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
    #[tokio::test]
    async fn test_http_auth_download() -> SpecResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/wpflow")
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
            then.status(200)
                .body("mock_binary_data");
        });

        // 2. 执行下载
        let temp_dir = tempfile::tempdir().owe_res()?;
        let http_addr = HttpAddr::from(server.url("/wpflow")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr
            .update_local(&temp_dir.path().to_path_buf())
            .await?;

        // 3. 验证结果
        assert!(temp_dir.path().join("wpflow").exists());
        mock.assert();
        Ok(())
    }
    #[ignore = "need more time"]
    #[tokio::test]
    async fn test_http_addr() -> SpecResult<()> {
        let path = PathBuf::from("/tmp");
        let addr = HttpAddr::from("https://dy-sec-generic.pkg.coding.net/sec-hub/generic/warp-flow/wpflow?version=1.0.89-alpha")
            .with_credentials(
                "generic-1747535977632",
                "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
            );
        addr.update_local(&path).await?;
        Ok(())
    }
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
}
