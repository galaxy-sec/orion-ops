use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use serde_derive::{Deserialize, Serialize};

use crate::{
    addr::{AddrType, GitAddr, LocalAddr},
    error::SpecResult,
    types::{AsyncUpdateable, UpdateOptions},
};
#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct DependItem {
    addr: AddrType,
    local: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    enable: Option<bool>,
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, Default)]
pub struct DependVec {
    local_root: PathBuf,
    depends: Vec<DependItem>,
}

impl DependVec {
    pub fn example() -> Self {
        let depends = vec![
            DependItem {
                addr: AddrType::from(LocalAddr::from("./example/data")),
                local: PathBuf::from("env_res"),
                rename: Some("mysql2".to_string()),
                enable: Some(false),
            },
            DependItem {
                addr: AddrType::from(GitAddr::from("https://github.com/xxx")),
                local: PathBuf::from("env_res"),
                rename: Some("mylib".to_string()),
                enable: Some(false),
            },
        ];

        DependVec {
            depends,
            local_root: PathBuf::from("./depends"),
        }
    }
    pub fn for_test() -> Self {
        let depends = vec![DependItem {
            addr: AddrType::from(LocalAddr::from("./example/knowlege/mysql")),
            local: PathBuf::from("env_res"),
            rename: Some("mysql_x86".to_string()),
            enable: Some(true),
        }];

        DependVec {
            depends,
            local_root: PathBuf::from("./depends"),
        }
    }
    pub async fn update(&self) -> SpecResult<()> {
        let options = UpdateOptions::for_depend();
        for dep in self.depends().iter() {
            if dep.is_enable() {
                dep.update(&self.local_root(), &options).await?;
            }
        }
        Ok(())
    }
    pub fn push(&mut self, item: DependItem) {
        self.depends.push(item);
    }
    pub fn check_exists(&self) -> Result<(), PathBuf> {
        for x in &self.depends {
            let path = self.local_root().join(x.local());
            if !path.exists() {
                return Err(path.clone());
            }
        }
        Ok(())
    }
}

impl DependItem {
    pub fn new(addr: AddrType, local: PathBuf) -> Self {
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
impl AsyncUpdateable for DependItem {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        self.addr.update_local(path, options).await
    }
}

impl DependItem {
    pub async fn update(&self, root: &PathBuf, options: &UpdateOptions) -> SpecResult<PathBuf> {
        //let item_path = path.join(self.local());
        let path = root.join(self.local());
        if let Some(rename) = self.rename() {
            self.update_rename(&path, rename, options).await
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

    use crate::{
        addr::{AddrType, LocalAddr},
        module::depend::{DependItem, DependVec},
        types::UpdateOptions,
    };

    #[tokio::test]
    async fn test_depend() {
        let prj_path = PathBuf::from("./test/temp/depend/");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("remove dir");
        }
        std::fs::create_dir_all(&prj_path).assert("create prj_path");
        let item = DependItem::new(
            AddrType::from(LocalAddr::from("./example/knowlege/mysql")),
            "env_res".into(),
        )
        .with_rename("mysql2");
        item.update(&prj_path, &UpdateOptions::for_test())
            .await
            .assert("update");
        assert!(prj_path.join("env_res").join("mysql2").exists())
    }

    #[test]
    fn test_serialize_to_yaml() {
        let item = DependItem {
            addr: AddrType::from(LocalAddr::from("./example/knowlege/mysql")),
            local: PathBuf::from("env_res"),
            rename: Some("mysql2".to_string()),
            enable: Some(true),
        };

        let vec = DependVec {
            depends: vec![item.clone(), item],
            local_root: PathBuf::from("./"),
        };
        let yaml_vec = serde_yaml::to_string(&vec).unwrap();
        println!("{:#}", yaml_vec);
        assert!(yaml_vec.contains("- addr:"));
        assert!(yaml_vec.contains("rename: mysql2"));
    }
}
