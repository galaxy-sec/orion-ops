use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    addr::{AddrType, path_file_name},
    const_vars::CONFS_DIR,
    error::SpecResult,
    types::{AsyncUpdateable, Configable},
};
use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use serde_derive::{Deserialize, Serialize};
#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct ConfSpec {
    version: String,
    files: Vec<ConfFile>,
}

#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct ConfFile {
    path: String,
    addr: Option<AddrType>,
}

#[derive(Getters, Clone, Debug, Serialize)]
pub struct ConfSpecRef {
    path: String,
    #[serde(skip_serializing)] // 序列化时跳过
    obj: ConfSpec,
}

impl ConfSpecRef {
    pub fn files(&self) -> &Vec<ConfFile> {
        self.obj.files()
    }
}

impl<'de> serde::Deserialize<'de> for ConfSpecRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 定义临时结构体用于反序列化
        #[derive(Deserialize)]
        struct RawRef {
            path: String,
        }
        // 先执行标准反序列化
        let raw = RawRef::deserialize(deserializer)?;
        // 构建实例
        let config = ConfSpecRef {
            obj: ConfSpecRef::load_ref(raw.path.as_str()).unwrap(),
            path: raw.path,
        };
        Ok(config)
    }
}

impl ConfSpecRef {
    pub fn new<S: Into<String>>(path: S) -> SpecResult<Self> {
        let path = path.into();
        let file_path = PathBuf::from(path.as_str());
        let obj = ConfSpec::from_conf(&file_path)?;
        Ok(Self { path, obj })
    }
    fn load_ref(path: &str) -> SpecResult<ConfSpec> {
        let path = PathBuf::from(path);
        ConfSpec::from_conf(&path)
    }
}

impl ConfFile {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            addr: None,
        }
    }
    pub fn with_addr<A: Into<AddrType>>(mut self, addr: A) -> Self {
        self.addr = Some(addr.into());
        self
    }
}
impl ConfSpec {
    pub fn try_load(path: &PathBuf) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load conf spec");
        ctx.with("path", format!("path: {}", path.display()));
        let file_content = fs::read_to_string(path).owe_conf().with(&ctx)?;
        let loaded: Self = toml::from_str(file_content.as_str())
            .owe_conf()
            .with(&ctx)?;
        Ok(loaded)
    }
    pub fn save(&self, path: &PathBuf) -> SpecResult<()> {
        let mut ctx = WithContext::want("save conf spec");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = toml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }

    pub fn new<S: Into<String>>(version: S) -> Self {
        Self {
            version: version.into(),
            files: Vec::new(),
        }
    }
    pub fn add(&mut self, file: ConfFile) {
        self.files.push(file);
    }
    pub fn from_files(values: Vec<&str>) -> Self {
        let mut ins = ConfSpec::new("1.0");
        for item in values {
            ins.add(ConfFile::new(item));
        }
        ins
    }
}

#[async_trait]
impl AsyncUpdateable for ConfSpec {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        let root = path.join(CONFS_DIR);
        std::fs::create_dir_all(&root).owe_res()?;
        for f in &self.files {
            if let Some(addr) = f.addr() {
                let filename = path_file_name(&PathBuf::from(f.path.as_str()))?;
                addr.update_rename(&root, filename.as_str()).await?;
            }
        }
        Ok(root)
    }
}

#[cfg(test)]
mod tests {
    use crate::addr::{HttpAddr, LocalAddr};

    use super::*;
    use httpmock::{Method::GET, MockServer};
    use orion_error::TestAssert;
    use tempfile::env::temp_dir;
    use tokio::fs;

    #[test]
    fn test_conf_spec_new() {
        let spec = ConfSpec::new("1.0");
        assert_eq!(spec.version(), "1.0");
        assert!(spec.files().is_empty());
    }

    #[test]
    fn test_conf_file_creation() {
        let file = ConfFile::new("config.yml");
        assert_eq!(file.path(), "config.yml");
        assert!(file.addr().is_none());

        let with_addr = file.with_addr(AddrType::Local(LocalAddr::from("/tmp")));
        assert!(with_addr.addr().is_some());
    }
    #[tokio::test]
    async fn test_async_update() -> SpecResult<()> {
        let src_dir = PathBuf::from("./temp/src");
        let dst_dir = PathBuf::from("./temp/dst");

        // 创建带地址的配置
        let mut spec = ConfSpec::new("3.0");
        spec.add(
            ConfFile::new("db.yml")
                .with_addr(AddrType::Local(LocalAddr::from("./temp/src/db.yml"))),
        );

        // 模拟本地文件

        fs::create_dir_all(&src_dir).await.owe_res()?;
        fs::create_dir_all(&dst_dir).await.owe_res()?;
        fs::write(src_dir.join("db.yml"), "[database]\nurl=\"localhost\"")
            .await
            .owe_res()?;

        // 执行更新
        let _ = spec.update_local(&dst_dir).await?;
        assert!(dst_dir.join("confs/db.yml").exists());

        // 清理
        fs::remove_dir_all(dst_dir).await.owe_res()?;
        fs::remove_dir_all(src_dir).await.owe_res()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_conf_with_http_addr() -> SpecResult<()> {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/global.yml");
            then.status(200).body("[settings]\nenv=\"test\"");
        });

        // 创建包含HttpAddr的配置
        let mut conf = ConfSpec::new("1.0");
        conf.add(ConfFile::new("remote.yml").with_addr(HttpAddr::from(server.url("/global.yml"))));

        // 测试更新
        //let src_dir = PathBuf::from("./temp/src");
        //let dst_dir = PathBuf::from("./temp/dst");
        let temp_dir = temp_dir();
        let updated_path = conf.update_local(&temp_dir).await.assert();

        // 验证下载的文件
        let content = fs::read_to_string(updated_path.join("remote.yml"))
            .await
            .owe_res()
            .with(format!("path: {}", updated_path.display()))?;
        assert!(content.contains("env=\"test\""));
        //fs::remove_dir_all(dst_dir).await.owe_res()?;
        Ok(())
    }
}
