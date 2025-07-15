use crate::addr::AddrType;
use crate::predule::SpecResult;
use crate::predule::{error, info};
use crate::types::AsyncUpdateable;
use crate::types::ResourceUpload;
use crate::types::UpdateValue;
use crate::update::UpdateOptions;
use derive_getters::Getters;
use derive_more::From;
use orion_error::ErrorOwe;
use orion_error::StructError;
use orion_error::UvsResFrom;
use serde_derive::{Deserialize, Serialize};
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OsType {
    MacOs,
    Ubuntu,
}
//produce addr
//deploy addr
//translate addr
//release_source
//deploy_source
//transfrom_addr
#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    name: String,
    #[serde(alias = "addr")]
    deployment_repo: AddrType,
    transit_storage: Option<AddrType>,
    release_repo: Option<AddrType>,
    local: String,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize, From, Default)]
#[serde(transparent)]
pub struct ArtifactPackage {
    items: Vec<Artifact>,
}
impl Deref for ArtifactPackage {
    type Target = Vec<Artifact>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
impl DerefMut for ArtifactPackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl Artifact {
    pub fn new<S: Into<String>, A: Into<AddrType>>(name: S, addr: A, local: S) -> Self {
        Self {
            name: name.into(),
            deployment_repo: addr.into(),
            transit_storage: None,
            release_repo: None,
            local: local.into(),
        }
    }

    // 直接从远程仓库下载
    pub async fn save_deployment(
        &self,
        dest_path: &Path,
        options: &UpdateOptions,
    ) -> SpecResult<UpdateValue> {
        std::fs::create_dir_all(dest_path).owe_res()?;
        let result = self
            .deployment_repo
            .update_rename(dest_path, &self.name, options)
            .await?;
        Ok(result)
    }

    // 将 release_repo 上的资源下载到 transit_storage
    pub async fn save_release_to_transit(
        &self,
        options: &UpdateOptions,
    ) -> SpecResult<UpdateValue> {
        if let Some(AddrType::Local(local)) = self.transit_storage() {
            let local_path = Path::new(local.path());
            std::fs::create_dir_all(local_path).owe_res()?;
            let result = if let Some(release) = self.release_repo() {
                release
                    .update_rename(local_path, &self.name, options)
                    .await?
            } else {
                UpdateValue::from(local_path.to_path_buf())
            };
            Ok(result)
        } else {
            Err(StructError::from_res("Unsupported Transit type".into()))
        }
    }

    // 将 transit_storage 上的资源上传到 deployment_repo
    pub async fn upload_transit_to_deployment(
        &self,
        options: &UpdateOptions,
    ) -> SpecResult<UpdateValue> {
        if let Some(AddrType::Local(local)) = self.transit_storage() {
            let path = Path::new(local.path()).join(self.name());
            if !path.exists() {
                return Err(StructError::from_res(format!(
                    "{} path not exist",
                    local.path()
                )));
            }
            let result = self.deployment_repo.upload_from(&path, options).await?;
            // 上传成功后删除原始内容
            let remove_status = if path.is_file() {
                std::fs::remove_file(path)
            } else {
                std::fs::remove_dir_all(path)
            };
            match remove_status {
                Ok(_) => info!("{} local file delete Success!", local.path()),
                Err(e) => error!("{} local file delete Failed, {}", local.path(), e),
            }
            Ok(result)
        } else {
            Err(StructError::from_res("Unsupported Transit type".into()))
        }
    }
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct DockImage {
    cep: String,
    addr: AddrType,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct BinPackage {
    cep: String,
    addr: AddrType,
}

#[cfg(test)]
mod tests {

    use home::home_dir;
    use orion_error::TestAssert;

    use super::*;
    use crate::addr::{GitAddr, HttpAddr, LocalAddr};

    #[tokio::test]
    async fn test_http_artifact_v1() -> SpecResult<()> {
        let artifact = Artifact::new(
            "galaxy-init",
            HttpAddr::from(
                "https://dy-sec-generic.pkg.coding.net/galaxy-open/generic/galaxy-init.sh?version=latest",
            ),
            "galaxy-init",
        );
        let path = home_dir()
            .unwrap_or("UNKOWN".into())
            .join(".cache")
            .join("v1");
        artifact
            .save_deployment(&path, &UpdateOptions::default())
            .await?;

        assert!(path.join("galaxy-init").exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_http_artifact_v2() -> SpecResult<()> {
        let home_dir = home_dir().assert();
        let transit_path = home_dir.join("transit");

        let release_type = AddrType::Http(HttpAddr::from(
            "https://dy-sec-generic.pkg.coding.net/galaxy-open/generic/galaxy-init.sh?version=latest",
        ));
        let transit_type = AddrType::Local(LocalAddr::from(transit_path.to_str().assert()));
        let deploy_type = AddrType::Git(GitAddr::from(
            "git@e.coding.net:dy-sec/practice/spec_git_test.git",
        ));
        let artifact = Artifact {
            name: "galaxy-init".to_string(),
            deployment_repo: deploy_type,
            transit_storage: Some(transit_type),
            release_repo: Some(release_type),
            local: "galaxy-init".to_string(),
        };
        artifact
            .save_release_to_transit(&UpdateOptions::default())
            .await?;
        artifact
            .upload_transit_to_deployment(&UpdateOptions::default())
            .await?;
        Ok(())
    }
}
