use getset::Getters;
use getset::Setters;
use getset::WithSetters;
use orion_error::ErrorOwe;
use orion_variate::addr::AddrResult;
use orion_variate::addr::Address;
use orion_variate::types::ResourceDownloader;
use orion_variate::types::UpdateUnit;
use orion_variate::update::DownloadOptions;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

use crate::types::Accessor;

#[derive(Getters, Clone, Debug, Deserialize, Serialize, Setters, WithSetters)]
#[getset(get = "pub")]
pub struct Artifact {
    name: String,
    version: String,
    #[serde(alias = "addr")]
    origin_addr: Address,
    #[getset(set_with = "pub", set = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_addr: Option<Address>,
    #[getset(set_with = "pub", set = "pub")]
    #[serde(default = "default_cache_enable")]
    cache_enable: bool,
    local: String,
}
fn default_cache_enable() -> bool {
    false
}

impl Artifact {
    pub fn new<S: Into<String>, A: Into<Address>>(name: S, version: S, addr: A, local: S) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            origin_addr: addr.into(),
            cache_addr: None,
            cache_enable: false,
            local: local.into(),
        }
    }

    // 直接从远程仓库下载
    pub async fn deploy_repo_to_local(
        &self,
        accessor: Accessor,
        dest_path: &Path,
        options: &DownloadOptions,
    ) -> AddrResult<UpdateUnit> {
        std::fs::create_dir_all(dest_path).owe_res()?;
        let result = accessor
            .download_rename(self.origin_addr(), dest_path, &self.name, options)
            .await?;
        Ok(result)
    }
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct DockImage {
    cep: String,
    addr: Address,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct BinPackage {
    cep: String,
    addr: Address,
}

#[cfg(test)]
mod tests {

    use home::home_dir;

    use orion_variate::addr::{GitRepository, HttpResource, accessor::GitAccessor};

    use crate::accessor::accessor_for_test;

    use super::*;

    #[ignore = "not run in ci"]
    #[tokio::test]
    async fn test_http_artifact_v1() -> AddrResult<()> {
        let artifact = Artifact::new(
            "hello-word",
            "0.1.0",
            HttpResource::from("https://github.com/galaxy-sec/hello-word.git"),
            "hello-word",
        );
        let path = home_dir()
            .unwrap_or("UNKOWN".into())
            .join(".cache")
            .join("v1");
        let accessor = accessor_for_test();
        artifact
            .deploy_repo_to_local(accessor, &path, &DownloadOptions::default())
            .await?;

        assert!(path.join("hello-word").exists());
        Ok(())
    }

    #[ignore = "not run in ci"]
    #[tokio::test]
    async fn test_http_artifact_v2() -> AddrResult<()> {
        let cache_addr = Address::Http(HttpResource::from(
            "https://dy-sec-generic.pkg.coding.net/galaxy-open/generic/galaxy-init.sh?version=latest",
        ));
        let deploy_type = Address::Git(
            GitRepository::from("git@github.com:galaxy-sec/spec_test.git").with_branch("main"),
        );
        let _artifact = Artifact {
            name: "galaxy-init".to_string(),
            version: "0.1.0".to_string(),
            origin_addr: deploy_type,
            cache_addr: Some(cache_addr),
            cache_enable: false,
            local: "galaxy-init".to_string(),
        }
        .with_cache_enable(true);
        Ok(())
    }
}
