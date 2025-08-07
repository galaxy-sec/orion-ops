use crate::{predule::*, types::Updateable};

use async_trait::async_trait;
use orion_variate::{
    addr::{AddrResult, Address, GitRepository, LocalPath, types::PathTemplate},
    types::{ResourceDownloader, UpdateUnit},
    update::DownloadOptions,
};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    addr: Address,
    local: PathTemplate,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    enable: Option<bool>,
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Default)]
pub struct DependencySet {
    dep_root: PathTemplate,
    deps: Vec<Dependency>,
}

impl DependencySet {
    pub fn example() -> Self {
        let depends = vec![
            Dependency {
                addr: Address::from(LocalPath::from("./example/data")),
                local: PathTemplate::from("env_res".to_string()),
                rename: Some("mysql2".to_string()),
                enable: Some(false),
            },
            Dependency {
                addr: Address::from(GitRepository::from("https://github.com/xxx")),
                local: PathTemplate::from("env_res".to_string()),
                rename: Some("mylib".to_string()),
                enable: Some(false),
            },
        ];

        DependencySet {
            deps: depends,
            dep_root: PathTemplate::from("./depends".to_string()),
        }
    }
    pub fn for_test() -> Self {
        let depends = vec![Dependency {
            addr: Address::from(LocalPath::from("./example/knowlege/mysql")),
            local: PathTemplate::from("env_res".to_string()),
            rename: Some("mysql_x86".to_string()),
            enable: Some(true),
        }];

        DependencySet {
            deps: depends,
            dep_root: PathTemplate::from("./depends".to_string()),
        }
    }
    pub fn push(&mut self, item: Dependency) {
        self.deps.push(item);
    }
}
impl Updateable<UpdateUnit> for DependencySet {
    async fn update_to_local(
        &self,
        accessor: &impl ResourceDownloader,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<UpdateUnit> {

        for dep in self.deps().iter() {
            if dep.is_enable() {
                dep.update_to_local( accessor,&self.dep_root().path(options.values()), options)
                    .await?;
            }
        }
        Ok(())
    }

impl Dependency {
    pub fn new(addr: Address, local: PathTemplate) -> Self {
        Self {
            addr,
            local,
            rename: None,
            enable: None,
        }
    }
    pub fn with_rename<S: Into<String>>(mut self, name: S) -> Self {
        self.rename = Some(name.into());
        self
    }
}

#[async_trait]
impl Updateable<UpdateUnit> for Dependency {
    async fn update_to_local(
        &self,
        accessor: &impl ResourceDownloader,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<UpdateUnit> {
        let path = path.join(self.local().path(options.values()));
        if let Some(rename) = self.rename() {
            accessor
                .download_rename(self.addr(), &path, rename, options)
                .await
        } else {
            accessor
                .download_to_local(self.addr(), &path, options)
                .await
        }
        //todo!();
        //self.addr.update_local(path, options).await
    }
}

impl Dependency {
    pub fn is_enable(&self) -> bool {
        self.enable.unwrap_or(true)
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;
    use orion_variate::{
        addr::{Address, LocalPath},
        update::DownloadOptions,
    };

    use crate::module::depend::{Dependency, DependencySet, PathTemplate};

    #[tokio::test]
    async fn test_depend() {
        let prj_path = PathBuf::from("./test_data/temp/depend/");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("remove dir");
        }
        std::fs::create_dir_all(&prj_path).assert("create prj_path");
        let item = Dependency::new(
            Address::from(LocalPath::from("./example/knowlege/mysql")),
            PathTemplate::from("env_res".to_string()),
        )
        .with_rename("mysql2");
        item.update(&prj_path, &DownloadOptions::for_test())
            .await
            .assert("update");
        assert!(prj_path.join("env_res").join("mysql2").exists())
    }

    #[test]
    fn test_serialize_to_yaml() {
        let item = Dependency {
            addr: Address::from(LocalPath::from("./example/knowlege/mysql")),
            local: PathTemplate::from("env_res".to_string()),
            rename: Some("mysql2".to_string()),
            enable: Some(true),
        };

        let vec = DependencySet {
            deps: vec![item.clone(), item],
            dep_root: PathTemplate::from("./".to_string()),
        };
        let yaml_vec = serde_yaml::to_string(&vec).unwrap();
        println!("{yaml_vec:#}",);
        assert!(yaml_vec.contains("- addr:"));
        assert!(yaml_vec.contains("rename: mysql2"));
    }
}
