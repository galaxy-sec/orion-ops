use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use orion_error::{ErrorOwe, ErrorWith};
use serde_derive::{Deserialize, Serialize};

use crate::{
    addr::AddrType,
    error::SpecResult,
    types::{AsyncUpdateable, Localizable, Persistable},
};

use super::{TargetNode, spec::ModuleSpec};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleSpecRef {
    name: String,
    addr: AddrType,
    node: TargetNode,
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
        }
    }
    pub fn with_effective(mut self, effective: bool) -> Self {
        self.effective = Some(effective);
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
    pub async fn update(&self) -> SpecResult<()> {
        if self.effective.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                std::fs::create_dir_all(local).owe_res().with(local)?;
                let _spec_path = self.addr.update_local(local).await?;
                let mod_path = local.join(self.name.as_str());
                let mut spec = ModuleSpec::load_from(&mod_path)?;
                let _x = spec.update_local(&mod_path).await?;
                spec.clean_other(self.node())?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModuleSpecRef {
    async fn localize(&self, dst_path: Option<PathBuf>) -> SpecResult<()> {
        if self.effective.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                let mod_path = local.join(self.name.as_str());
                let spec = ModuleSpec::load_from(&mod_path)?;
                let cur_dst_path = dst_path.map(|x| x.join(self.name.as_str()));
                spec.localize(cur_dst_path).await?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}
