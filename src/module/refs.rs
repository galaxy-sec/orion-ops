use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use log::{debug, error, info};
use orion_error::{ErrorOwe, ErrorWith};
use serde_derive::{Deserialize, Serialize};

use crate::{
    addr::AddrType,
    error::SpecResult,
    types::{AsyncUpdateable, Localizable, LocalizePath, Persistable, UpdateLevel, UpdateOptions},
};

use super::{TargetNode, spec::ModuleSpec};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleSpecRef {
    name: String,
    addr: AddrType,
    node: TargetNode,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    enable: Option<bool>,
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
            enable: None,
            local: None,
        }
    }
    pub fn with_enable(mut self, effective: bool) -> Self {
        self.enable = Some(effective);
        self
    }

    pub fn is_enable(&self) -> bool {
        self.enable.unwrap_or(true)
    }
    pub fn spec_path(&self, root: &Path) -> PathBuf {
        root.join("mods").join(self.name.as_str())
    }
    pub fn set_local(&mut self, local: PathBuf) {
        self.local = Some(local);
    }
}
impl ModuleSpecRef {
    pub async fn update(&self, _sys_root: &Path, options: &UpdateOptions) -> SpecResult<()> {
        //trace!(target: "spec/mod/",  "{:?}",self );
        if self.is_enable() {
            if let Some(local) = &self.local {
                let mut flag = log_guard!(
                    info!(target: "/mod/ref",  "update mod ref {} success!", self.name ),
                    error!(target: "/mod/ref", "update mod ref {} fail!", self.name )
                );
                std::fs::create_dir_all(local).owe_res().with(local)?;
                if options.level() != UpdateLevel::Elm {
                    let _spec_path = self.addr.update_local(local, options).await?;
                }
                debug!(target: "mod/ref",  "update target success!" );
                let mod_path = local.join(self.name.as_str());
                let mut spec = ModuleSpec::load_from(&mod_path).with(&mod_path)?;
                let _x = spec.update_local(&mod_path, options).await?;
                spec.clean_other(self.node())?;
                flag.flag_suc();
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Localizable for ModuleSpecRef {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()> {
        if self.enable.is_none_or(|x| x) {
            if let Some(local) = &self.local {
                let mut flag = log_guard!(
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
