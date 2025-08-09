use super::predule::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    const_vars::CONFS_DIR,
    error::MainResult,
    types::{Accessor, RefUpdateable},
};
use async_trait::async_trait;
use orion_common::serde::Configable;
use orion_error::ErrorConv;
use orion_infra::auto_exit_log;
use orion_variate::{
    addr::{Address, accessor::path_file_name},
    types::{ResourceDownloader, UpdateUnit},
    update::DownloadOptions,
};
// 由于 `crate::tools::log_flag` 未定义，移除该导入
#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct ConfSpec {
    version: String,
    #[serde(default = "default_local_root")]
    local_root: String,
    files: Vec<ConfFile>,
}
fn default_local_root() -> String {
    CONFS_DIR.to_string()
}

#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct ConfFile {
    path: String,
    addr: Option<Address>,
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
    pub fn new<S: Into<String>>(path: S) -> MainResult<Self> {
        let path = path.into();
        let file_path = PathBuf::from(path.as_str());
        let obj = ConfSpec::from_conf(&file_path).owe_conf()?;
        Ok(Self { path, obj })
    }
    fn load_ref(path: &str) -> MainResult<ConfSpec> {
        let path = PathBuf::from(path);
        ConfSpec::from_conf(&path).owe_conf()
    }
}

impl ConfFile {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            addr: None,
        }
    }
    pub fn with_addr<A: Into<Address>>(mut self, addr: A) -> Self {
        self.addr = Some(addr.into());
        self
    }
}
impl ConfSpec {
    pub fn save(&self, path: &PathBuf) -> MainResult<()> {
        let mut ctx = WithContext::want("save conf spec");
        ctx.with("path", format!("path: {}", path.display()));
        let data_content = toml::to_string(self).owe_data().with(&ctx)?;
        fs::write(path, data_content).owe_res().with(&ctx)?;
        Ok(())
    }

    pub fn new<S: Into<String>>(version: S, local_root: S) -> Self {
        Self {
            version: version.into(),
            local_root: local_root.into(),
            files: Vec::new(),
        }
    }
    pub fn add(&mut self, file: ConfFile) {
        self.files.push(file);
    }
    pub fn default_from_files(values: Vec<&str>) -> Self {
        let mut ins = ConfSpec::new("1.0", CONFS_DIR);
        for item in values {
            ins.add(ConfFile::new(item));
        }
        ins
    }
}

#[async_trait]
impl RefUpdateable<UpdateUnit> for ConfSpec {
    async fn update_local(
        &self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<UpdateUnit> {
        debug!( target:"spec/confspec", "upload_local confspec begin: {}" ,path.display() );

        let mut is_suc = auto_exit_log!(
            info!( target:"spec/confspec", "upload_local confspec suc: {}" ,path.display() ),
            error!( target:"spec/confspec", "upload_local confspec fail: {}" ,path.display() )
        );
        let root = path.join(self.local_root());
        std::fs::create_dir_all(&root).owe_res()?;
        for f in &self.files {
            if let Some(addr) = f.addr() {
                let filename = path_file_name(&PathBuf::from(f.path.as_str())).err_conv()?;

                let x = accessor
                    .download_rename(addr, &root, filename.as_str(), options)
                    .await
                    .err_conv()?;
                is_suc.mark_suc();
                return Ok(x);
            }
        }
        Ok(UpdateUnit::from(root))
    }
}

#[cfg(test)]
mod tests {

    use crate::accessor::accessor_for_test;

    use super::*;
    use httpmock::{Method::GET, MockServer};
    use orion_error::TestAssert;
    use orion_variate::{
        addr::{HttpResource, LocalPath},
        tools::test_init,
    };
    use tokio::fs;

    #[test]
    fn test_conf_spec_new() {
        let spec = ConfSpec::new("1.0", CONFS_DIR);
        assert_eq!(spec.version(), "1.0");
        assert!(spec.files().is_empty());
    }

    #[test]
    fn test_conf_file_creation() {
        let file = ConfFile::new("config.yml");
        assert_eq!(file.path(), "config.yml");
        assert!(file.addr().is_none());

        let with_addr = file.with_addr(Address::Local(LocalPath::from("/tmp")));
        assert!(with_addr.addr().is_some());
    }
    #[tokio::test]
    async fn test_async_update() -> MainResult<()> {
        test_init();
        let src_dir = PathBuf::from("./temp/src");
        let dst_dir = PathBuf::from("./temp/dst");

        // 创建带地址的配置
        let mut spec = ConfSpec::new("3.0", CONFS_DIR);
        spec.add(
            ConfFile::new("db.yml").with_addr(Address::Local(LocalPath::from("./temp/src/db.yml"))),
        );

        // 模拟本地文件

        fs::create_dir_all(&src_dir).await.owe_res()?;
        fs::create_dir_all(&dst_dir).await.owe_res()?;
        fs::write(src_dir.join("db.yml"), "[database]\nurl=\"localhost\"")
            .await
            .owe_res()?;

        let accessor = accessor_for_test();
        // 执行更新
        let _ = spec
            .update_local(accessor, &dst_dir, &DownloadOptions::for_test())
            .await
            .owe_logic()?;
        assert!(dst_dir.join("confs/db.yml").exists());

        // 清理
        fs::remove_dir_all(dst_dir).await.owe_res()?;
        fs::remove_dir_all(src_dir).await.owe_res()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_conf_with_http_addr() -> MainResult<()> {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/global.yml");
            then.status(200).body("[settings]\nenv=\"test\"");
        });

        // 创建包含HttpResource的配置
        let mut conf = ConfSpec::new("1.0", CONFS_DIR);
        conf.add(
            ConfFile::new("remote.yml").with_addr(HttpResource::from(server.url("/global.yml"))),
        );

        // 测试更新
        //let src_dir = PathBuf::from("./temp/src");
        //let dst_dir = PathBuf::from("./temp/dst");
        //let temp_dir = temp_dir();
        let temp_dir = PathBuf::from("./test_data/temp/http");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).assert();
        }
        std::fs::create_dir_all(&temp_dir).assert();

        let accessor = accessor_for_test();
        let updated_v = conf
            .update_local(accessor, &temp_dir, &DownloadOptions::for_test())
            .await
            .assert();

        assert_eq!(
            updated_v.position(),
            &temp_dir.join(CONFS_DIR).join("remote.yml")
        );
        // 验证下载的文件
        let content = fs::read_to_string(updated_v.position())
            .await
            .owe_res()
            .with(format!("path: {}", updated_v.position().display()))?;
        assert!(content.contains("env=\"test\""));
        //fs::remove_dir_all(dst_dir).await.owe_res()?;
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn test_conf_with_addr_addr() -> MainResult<()> {
        // 创建包含HttpResource的配置
        let mut conf = ConfSpec::new("1.0", CONFS_DIR);
        conf.add(ConfFile::new("bitnami").with_addr(HttpResource::from(
            "https://github.com/galaxy-sec/hello-word.git",
        )));

        // 测试更新
        //let src_dir = PathBuf::from("./temp/src");
        //let dst_dir = PathBuf::from("./temp/dst");
        //let temp_dir = temp_dir();
        let temp_dir = PathBuf::from("./test_data/temp/conf_dst");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).assert();
        }
        std::fs::create_dir_all(&temp_dir).assert();
        let accessor = accessor_for_test();
        let updated_v = conf
            .update_local(accessor, &temp_dir, &DownloadOptions::for_test())
            .await
            .assert();
        assert_eq!(
            updated_v.position(),
            &temp_dir.join(CONFS_DIR).join("bitnami")
        );

        Ok(())
    }
}
