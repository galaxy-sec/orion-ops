use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use serde_derive::{Deserialize, Serialize};

use crate::{
    addr::AddrType,
    error::SpecResult,
    types::{AsyncUpdateable, Persistable},
};

use super::{TargetNodeType, spec::ModuleSpec};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleSpecRef {
    name: String,
    addr: AddrType,
    node: TargetNodeType,
    effective: Option<bool>,
}

impl ModuleSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(
        name: S,
        addr: A,
        node: TargetNodeType,
    ) -> ModuleSpecRef {
        Self {
            name: name.into(),
            addr: addr.into(),
            node,
            effective: None,
        }
    }
    pub fn with_effective(mut self, effective: bool) -> Self {
        self.effective = Some(effective);
        self
    }

    pub fn is_effective(&self) -> bool {
        self.effective.is_none_or(|x| x)
    }
}
#[async_trait]
impl AsyncUpdateable for ModuleSpecRef {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        if self.effective.is_none_or(|x| x) {
            let spec_path = self.addr.update_local(path).await?;
            let mod_path = path.join(self.name.as_str());
            let mut spec = ModuleSpec::load_from(&mod_path)?;
            let _x = spec.update_local(&mod_path).await?;
            spec.clean_other(self.node())?;
            Ok(spec_path)
        } else {
            Ok(path.to_path_buf())
        }
    }
}
