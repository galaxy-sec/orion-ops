use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsResFrom};
use serde_derive::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::{error::SpecResult, types::AsyncUpdateable};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "http")]
pub struct HttpAddr {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    async fn update_local(&self, dest_dir: &Path) -> SpecResult<PathBuf> {
        use indicatif::{ProgressBar, ProgressStyle};

        let client = reqwest::Client::new();
        let mut request = client.get(&self.url);

        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            request = request.basic_auth(u, Some(p));
        }

        let mut response = request
            .send()
            .await
            .owe_res()
            .with(format!("Failed to download {}", self.url))?;

        if !response.status().is_success() {
            return Err(StructError::from_res(format!(
                "HTTP request failed: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let file_name = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("downloaded_file");

        let dest_path = dest_dir.join(file_name);
        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .owe_conf()
            .with(format!("Failed to create {}", dest_path.display()))?;

        // 创建进度条
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").owe_logic()?
            .progress_chars("#>-"));

        let mut downloaded: u64 = 0;

        while let Some(chunk) = response.chunk().await.owe_data()? {
            file.write_all(&chunk)
                .await
                .owe_sys()
                .with(format!("Failed to write to {}", dest_path.display()))?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("下载完成");
        Ok(dest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};

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
}
