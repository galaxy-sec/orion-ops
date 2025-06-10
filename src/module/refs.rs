use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use log::{error, info};
use orion_error::{ErrorOwe, ErrorWith};
use serde_derive::{Deserialize, Serialize};

use crate::{
    addr::AddrType,
    error::SpecResult,
    types::{AsyncUpdateable, Localizable, LocalizePath, Persistable, UpdateOptions},
};

use super::{TargetNode, spec::ModuleSpec};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct DependItem {
    addr: AddrType,
    local: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rename: Option<String>,
}

impl DependItem {
    pub fn new(addr: AddrType, local: PathBuf) -> Self {
        Self {
            addr,
            local,
            rename: None,
        }
    }
    pub fn with_rename<S: Into<String>>(mut self, name: S) -> Self {
        self.rename = Some(name.into());
        self
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleSpecRef {
    name: String,
    addr: AddrType,
    node: TargetNode,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    depends: Vec<DependItem>,
    effective: Option<bool>,
    #[serde(skip)]
    local: Option<PathBuf>,
}

impl ModuleSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(
        name: S,
        addr: A,
        node: TargetNode,
    ) -> ModuleSpecRef {
        Self {
            name: name.into(),
            addr: addr.into(),
            node,
            effective: None,
            local: None,
            depends: Vec::new(),
        }
    }
    pub fn with_effective(mut self, effective: bool) -> Self {
        self.effective = Some(effective);
        self
    }

    pub fn with_depend(mut self, depend: DependItem) -> Self {
        self.depends.push(depend);
        self
    }

    pub fn is_effective(&self) -> bool {
        self.effective.is_none_or(|x| x)
    }
    pub fn spec_path(&self, root: &Path) -> PathBuf {
        root.join("mods").join(self.name.as_str())
    }
    pub fn set_local(&mut self, local: PathBuf) {
        self.local = Some(local);
    }
}
impl ModuleSpecRef {
    pub async fn update(&self, sys_root: &Path, options: &UpdateOptions) -> SpecResult<()> {
        if self.effective.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                let mut flag = log_flag!(
                    info!(target: "spec/mod/", "update mod {} success!", self.name ),
                    error!(target: "spec/mod/", "update mod {} fail!", self.name )
                );
                std::fs::create_dir_all(local).owe_res().with(local)?;
                let _spec_path = self.addr.update_local(local, options).await?;
                for item in self.depends() {
                    item.update_local(sys_root, &UpdateOptions::for_depend())
                        .await?;
                }
                let mod_path = local.join(self.name.as_str());
                let mut spec = ModuleSpec::load_from(&mod_path)?;
                let _x = spec.update_local(&mod_path, options).await?;
                spec.clean_other(self.node())?;
                flag.flag_suc();
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AsyncUpdateable for DependItem {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        self.addr.update_local(path, options).await
    }
}

impl DependItem {
    pub async fn update(&self, options: &UpdateOptions) -> SpecResult<PathBuf> {
        //let item_path = path.join(self.local());
        let path = self.local();
        if let Some(rename) = self.rename() {
            self.update_rename(path, rename, options).await
        } else {
            self.update_local(path, options).await
        }
    }
}

#[async_trait]
impl Localizable for ModuleSpecRef {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()> {
        if self.effective.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                let mut flag = log_flag!(
                    info!(target: "spec/mod/", "localize mod {} success!", self.name ),
                    error!(target: "spec/mod/", "localize mod {} fail!", self.name )
                );
                let mod_path = local.join(self.name.as_str());
                let spec = ModuleSpec::load_from(&mod_path)?;
                if let Some(dst) = &dst_path {
                    spec.save_main(dst.local(), None)?;
                }
                let value = PathBuf::from(self.name());
                let local = PathBuf::from(self.name()).join("local");
                let cur_dst_path = dst_path.map(|x| x.join(local, value));
                spec.localize(cur_dst_path).await?;
                flag.flag_suc();
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        addr::{AddrType, LocalAddr},
        types::UpdateOptions,
    };

    use super::DependItem;
    #[tokio::test]
    async fn test_depend() {
        let prj_path = PathBuf::from("./test/temp/depend/");
        if prj_path.exists() {
            std::fs::remove_dir_all(&prj_path).assert("remove dir");
        }
        std::fs::create_dir_all(&prj_path).assert("create prj_path");
        let item = DependItem::new(
            AddrType::from(LocalAddr::from("./example/knowlege/mysql")),
            prj_path.join("env_res"),
        )
        .with_rename("mysql2");
        item.update(&UpdateOptions::for_test())
            .await
            .assert("update");
        assert!(prj_path.join("env_res").join("mysql2").exists())
    }
}
