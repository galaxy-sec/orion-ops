use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

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
}

impl DependItem {
    pub fn new(addr: AddrType, local: PathBuf) -> Self {
        Self { addr, local }
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
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AsyncUpdateable for DependItem {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        let target = path.join(self.local());
        self.addr.update_local(&target, options).await?;
        Ok(target)
    }
}

#[async_trait]
impl Localizable for ModuleSpecRef {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()> {
        if self.effective.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                let mod_path = local.join(self.name.as_str());
                let spec = ModuleSpec::load_from(&mod_path)?;
                if let Some(dst) = &dst_path {
                    spec.save_main(dst.local(), None)?;
                }
                let value = PathBuf::from(self.name());
                let local = PathBuf::from(self.name()).join("local");
                let cur_dst_path = dst_path.map(|x| x.join(local, value));
                spec.localize(cur_dst_path).await?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}
