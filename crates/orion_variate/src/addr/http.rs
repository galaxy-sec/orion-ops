use crate::{predule::*, types::RemoteUpdate, update::UpdateOptions, vars::EnvDict};

use orion_error::UvsResFrom;
use tokio::io::AsyncWriteExt;
use tracing::info;
use url::Url;

use crate::{types::LocalUpdate, vars::EnvEvalable};

use super::AddrResult;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "http")]
pub struct HttpAddr {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
}

impl EnvEvalable<HttpAddr> for HttpAddr {
    fn env_eval(self, dict: &EnvDict) -> HttpAddr {
        Self {
            url: self.url.env_eval(dict),
            username: self.username.env_eval(dict),
            password: self.password.env_eval(dict),
        }
    }
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
impl HttpAddr {
    pub fn get_filename(&self) -> Option<String> {
        let url = Url::parse(&self.url).ok()?;
        url.path_segments()?.next_back().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
    }
}

impl HttpAddr {
    pub async fn upload<P: AsRef<Path>>(&self, file_path: P, method: &str) -> AddrResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};
        let mut ctx = WithContext::want("upload url");

        let client = reqwest::Client::new();
        let file_name = self
            .get_filename()
            .unwrap_or_else(|| "file.bin".to_string());
        ctx.with_path("local file", file_path.as_ref());

        println!(
            "upload : {} => \n {}",
            file_path.as_ref().display(),
            self.url(),
        );
        let file_content = std::fs::read(file_path).owe_data().with(&ctx)?;
        // 记录本地文件大小
        println!("本地文件大小: {} 字节", file_content.len());
        // 创建进度条
        let content_len = file_content.len() as u64;
        let pb = ProgressBar::new(content_len);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").owe_logic()?
            .progress_chars("#>-"));

        ctx.with("url", self.url());
        let mut request = match method.to_uppercase().as_str() {
            "POST" => {
                let part = reqwest::multipart::Part::stream_with_length(file_content, content_len)
                    .file_name(file_name);
                let form = reqwest::multipart::Form::new().part("file", part);
                client.post(&self.url).multipart(form)
            }
            "PUT" => {
                // PUT方法直接使用文件内容作为请求体，避免multipart额外头部
                client.put(&self.url).body(file_content)
            }
            _ => {
                return Err(StructError::from_res(format!(
                    "Unsupported HTTP method: {}",
                    method
                )));
            }
        };

        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            request = request.basic_auth(u, Some(p));
        }

        let response = request.send().await.owe_res().with(&ctx)?;
        response.error_for_status().owe_res().with(&ctx)?;
        pb.finish_with_message("上传完成");
        Ok(())
    }

    pub async fn download(&self, dest_path: &Path, options: &UpdateOptions) -> AddrResult<PathBuf> {
        use indicatif::{ProgressBar, ProgressStyle};

        if dest_path.exists() && options.reuse_remote_file() {
            info!(target :"spec/addr", "{} exists , ignore!! ",dest_path.display());
            return Ok(dest_path.to_path_buf());
        }
        let mut ctx = WithContext::want("download url");
        ctx.with("url", self.url());
        let client = reqwest::Client::new();
        let mut request = client.get(&self.url);

        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            request = request.basic_auth(u, Some(p));
        }

        println!("donwload :{}", self.url());
        let mut response = request.send().await.owe_res().with(&ctx)?;

        if !response.status().is_success() {
            return Err(StructError::from_res(format!(
                "HTTP request failed: {}",
                response.status()
            )))
            .with(&ctx);
        }

        let total_size = response.content_length().unwrap_or(0);

        ctx.with_path("local", dest_path);
        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .owe_conf()
            .with(&ctx)?;

        // 创建进度条
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").owe_logic()?
            .progress_chars("#>-"));

        let mut downloaded: u64 = 0;

        while let Some(chunk) = response.chunk().await.owe_data().with(&ctx)? {
            file.write_all(&chunk).await.owe_sys().with(&ctx)?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("下载完成");
        Ok(dest_path.to_path_buf())
    }
}

#[async_trait]
impl LocalUpdate for HttpAddr {
    async fn update_local(
        &self,
        dest_dir: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let file = self.get_filename();
        let dest_path = dest_dir.join(file.unwrap_or("file.tmp".into()));
        Ok(UpdateUnit::from(self.download(&dest_path, options).await?))
    }
}

#[async_trait]
impl RemoteUpdate for HttpAddr {
    async fn update_remote(&self, path: &Path, _: &UpdateOptions) -> AddrResult<UpdateUnit> {
        if !path.exists() {
            return Err(StructError::from_res("path not exist".into()));
        }
        self.upload(path, "POST").await?;
        if path.is_file() {
            std::fs::remove_file(path).owe_res()?;
        } else {
            std::fs::remove_dir_all(path).owe_res()?;
        }
        Ok(UpdateUnit::from(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{addr::AddrResult, update::UpdateOptions};

    use super::*;
    use httpmock::{Method::GET, MockServer};

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_auth_download() -> AddrResult<()> {
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
            .update_local(temp_dir.path(), &UpdateOptions::for_test())
            .await?;

        // 3. 验证结果
        assert!(temp_dir.path().join("wpflow").exists());
        mock.assert();
        Ok(())
    }
    #[ignore = "need more time"]
    #[tokio::test(flavor = "current_thread")]
    async fn test_http_addr() -> AddrResult<()> {
        let path = PathBuf::from("/tmp");
        let addr = HttpAddr::from("https://dy-sec-generic.pkg.coding.net/sec-hub/generic/warp-flow/wpflow?version=1.0.89-alpha")
            .with_credentials(
                "generic-1747535977632",
                "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
            );
        addr.update_local(&path, &UpdateOptions::for_test()).await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_upload_post() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/upload")
                .header_exists("content-type")  // 检查 multipart 头
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
            then.status(200)
                .body("upload success");
        });

        // 2. 创建临时测试文件
        let temp_dir = tempfile::tempdir().owe_res()?;
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "test content")
            .await
            .owe_sys()?;

        // 3. 执行上传
        let http_addr = HttpAddr::from(server.url("/upload")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr.upload(&file_path, "POST").await?;

        // 4. 验证结果
        mock.assert();
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_upload_put() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::PUT)
                .path("/upload_put")
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM="); // 移除content-type检查，PUT请求不使用multipart
            then.status(200).body("upload success");
        });

        // 2. 创建临时测试文件
        let temp_dir = tempfile::tempdir().owe_res()?;
        let file_path = temp_dir.path().join("test_put.txt");
        tokio::fs::write(&file_path, "test put content")
            .await
            .owe_sys()?;

        // 3. 执行上传
        let http_addr = HttpAddr::from(server.url("/upload_put")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr.upload(&file_path, "PUT").await?;

        // 4. 验证结果
        mock.assert();
        Ok(())
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_get_filename_with_regular_url() {
        let addr = HttpAddr::from("http://example.com/file.txt");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_query_params() {
        let addr = HttpAddr::from("http://example.com/file.txt?version=1.0");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_fragment() {
        let addr = HttpAddr::from("http://example.com/file.txt#section1");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_multiple_path_segments() {
        let addr = HttpAddr::from("http://example.com/path/to/file.txt");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_trailing_slash() {
        let addr = HttpAddr::from("http://example.com/path/");
        assert_eq!(addr.get_filename(), None);
    }

    #[test]
    fn test_get_filename_with_empty_path() {
        let addr = HttpAddr::from("http://example.com");
        assert_eq!(addr.get_filename(), None);
    }

    #[test]
    fn test_get_filename_with_invalid_url() {
        let addr = HttpAddr::from("not a valid url");
        assert_eq!(addr.get_filename(), None);
    }

    #[test]
    fn test_get_filename_with_encoded_characters() {
        let addr = HttpAddr::from("http://example.com/file%20name.txt");
        assert_eq!(addr.get_filename(), Some("file%20name.txt".to_string()));
    }
}

#[cfg(test)]
mod test3 {
    use super::*;
    use httpmock::MockServer;

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_upload_post() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/upload")
                .header_exists("content-type")  // 检查 multipart 头
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
            then.status(200)
                .body("upload success");
        });

        // 2. 创建临时测试文件
        let temp_dir = tempfile::tempdir().owe_res()?;
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "test content")
            .await
            .owe_sys()?;

        // 3. 执行上传
        let http_addr = HttpAddr::from(server.url("/upload")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr
            .update_remote(&file_path, &UpdateOptions::for_test())
            .await?;

        // 4. 验证结果
        mock.assert();
        assert!(!file_path.exists());
        Ok(())
    }
}
