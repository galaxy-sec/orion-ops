use crate::predule::*;

use async_trait::async_trait;
use orion_variate::{
    addr::{AddrResult, Address, GitRepository, LocalPath},
    update::DownloadOptions,
};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    addr: Address,
    local: EnvVarPath,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    enable: Option<bool>,
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Default)]
pub struct DependencySet {
    dep_root: EnvVarPath,
    deps: Vec<Dependency>,
}

impl DependencySet {
    pub fn example() -> Self {
        let depends = vec![
            Dependency {
                addr: Address::from(LocalPath::from("./example/data")),
                local: EnvVarPath::from("env_res".to_string()),
                rename: Some("mysql2".to_string()),
                enable: Some(false),
            },
            Dependency {
                addr: Address::from(GitRepository::from("https://github.com/xxx")),
                local: EnvVarPath::from("env_res".to_string()),
                rename: Some("mylib".to_string()),
                enable: Some(false),
            },
        ];

        DependencySet {
            deps: depends,
            dep_root: EnvVarPath::from("./depends".to_string()),
        }
    }
    pub fn for_test() -> Self {
        let depends = vec![Dependency {
            addr: Address::from(LocalPath::from("./example/knowlege/mysql")),
            local: EnvVarPath::from("env_res".to_string()),
            rename: Some("mysql_x86".to_string()),
            enable: Some(true),
        }];

        DependencySet {
            deps: depends,
            dep_root: EnvVarPath::from("./depends".to_string()),
        }
    }
    pub async fn update(&self, options: &DownloadOptions) -> AddrResult<()> {
        //let options = DownloadOptions::for_depend();
        //options.
        for dep in self.deps().iter() {
            if dep.is_enable() {
                dep.update(&self.dep_root().path(options.values()), options)
                    .await?;
            }
        }
        Ok(())
    }
    pub fn push(&mut self, item: Dependency) {
        self.deps.push(item);
    }
}

impl Dependency {
    pub fn new(addr: Address, local: EnvVarPath) -> Self {
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
impl LocalUpdate for Dependency {
    async fn update_local(&self, path: &Path, options: &DownloadOptions) -> AddrResult<UpdateUnit> {
        self.addr.update_local(path, options).await
    }
}

impl Dependency {
    pub async fn update(&self, root: &Path, options: &DownloadOptions) -> AddrResult<UpdateUnit> {
        //let item_path = path.join(self.local());
        let path = root.join(self.local().path(options.values()));
        if let Some(rename) = self.rename() {
            self.update_local_rename(&path, rename, options).await
        } else {
            self.update_local(&path, options).await
        }
    }
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

    use crate::module::depend::{Dependency, DependencySet, EnvVarPath};

    #[tokio::test]
    async fn test_depend() {
        let prj_path = PathBuf::from("./test_data/temp/depend/");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("remove dir");
        }
        std::fs::create_dir_all(&prj_path).assert("create prj_path");
        let item = Dependency::new(
            Address::from(LocalPath::from("./example/knowlege/mysql")),
            EnvVarPath::from("env_res".to_string()),
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
            local: EnvVarPath::from("env_res".to_string()),
            rename: Some("mysql2".to_string()),
            enable: Some(true),
        };

        let vec = DependencySet {
            deps: vec![item.clone(), item],
            dep_root: EnvVarPath::from("./".to_string()),
        };
        let yaml_vec = serde_yaml::to_string(&vec).unwrap();
        println!("{yaml_vec:#}",);
        assert!(yaml_vec.contains("- addr:"));
        assert!(yaml_vec.contains("rename: mysql2"));
    }
}
