use crate::addr::AddrType;
use crate::predule::SpecResult;
use crate::predule::{debug, info};
use crate::types::AsyncUpdateable;
use crate::types::ResourceUpload;
use crate::types::UpdateValue;
use crate::update::UpdateOptions;
use derive_getters::Getters;
use derive_more::From;
use orion_error::ErrorOwe;
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

    pub async fn update_rename(
        self,
        dest_path: &Path,
        options: &UpdateOptions,
    ) -> SpecResult<UpdateValue> {
        let result;
        if let (Some(AddrType::Local(local)), Some(release)) =
            (self.transit_storage, self.release_repo)
        {
            std::fs::create_dir_all(&local.path()).owe_res()?;
            // 从release repo下载到 transit storage
            release
                .update_rename(Path::new(local.path()), &self.name, options)
                .await?;
            let transit_path = Path::new(local.path()).join(self.name);
            //  从 transit storage上传到 deployment repo
            let deployment_status = self
                .deployment_repo
                .upload_from(&transit_path, options)
                .await?;
            let remove_result = if transit_path.is_file() {
                std::fs::remove_file(transit_path)
            } else {
                std::fs::remove_dir_all(transit_path)
            };
            match remove_result {
                Ok(_) => info!("remove file success"),
                Err(e) => debug!("remove file failed: {}", e),
            }
            result = deployment_status;
        } else {
            std::fs::create_dir_all(&dest_path).owe_res()?;
            result = self
                .deployment_repo
                .update_rename(dest_path, &self.name, options)
                .await?;
        }
        Ok(result)
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
            "specGitTest",
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
            "postgresql-17.4.tar.gz",
        );
        let path = home_dir()
            .unwrap_or("UNKOWN".into())
            .join(".cache")
            .join("v1");
        artifact
            .update_rename(&path, &UpdateOptions::default())
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_http_artifact_v2() -> SpecResult<()> {
        let home_dir = home_dir().assert();
        let path = home_dir.join(".cache").join("v2");
        let transit_path = home_dir.join(".cache").join("transit");

        let release_type = AddrType::Http(HttpAddr::from(
            "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz",
        ));
        let transit_type = AddrType::Local(LocalAddr::from(transit_path.to_str().assert()));
        let deploy_type = AddrType::Git(GitAddr::from(
            "git@e.coding.net:dy-sec/practice/spec_git_test.git",
        ));
        let artifact = Artifact {
            name: "postgresql-17.4.tar".to_string(),
            deployment_repo: deploy_type,
            transit_storage: Some(transit_type),
            release_repo: Some(release_type),
            local: "postgresql-17.4.tar.gz".to_string(),
        };
        artifact
            .update_rename(&path, &UpdateOptions::default())
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_http_artifact_v3() -> SpecResult<()> {
        let home_dir = home_dir().assert();
        let path = home_dir.join(".cache").join("v2");
        let transit_path = home_dir.join(".cache").join("transit");

        let release_type = AddrType::Http(HttpAddr::from(
            "https://dy-sec-generic.pkg.coding.net/galaxy-open/ubuntu22/fluent-bit?version=4.0.2",
        ));
        let transit_type = AddrType::Local(LocalAddr::from(transit_path.to_str().assert()));
        let deploy_type = AddrType::Git(
            GitAddr::from("git@e.coding.net:dy-sec/practice/spec_git_test.git").branch("master"),
        );
        let artifact = Artifact {
            name: "fluent-bit".to_string(),
            deployment_repo: deploy_type,
            transit_storage: Some(transit_type),
            release_repo: Some(release_type),
            local: "fluent-bit".to_string(),
        };
        artifact
            .update_rename(&path, &UpdateOptions::default())
            .await?;
        Ok(())
    }
}
