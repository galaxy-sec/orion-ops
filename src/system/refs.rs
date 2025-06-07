use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use crate::{addr::AddrType, error::SpecResult, types::AsyncUpdateable};
use serde_derive::{Deserialize, Serialize};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysModelSpecRef {
    name: String,
    addr: AddrType,
}
impl SysModelSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(name: S, addr: A) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
        }
    }
}

#[async_trait]
impl AsyncUpdateable for SysModelSpecRef {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        self.addr.update_local(path).await
    }

    async fn update_rename(&self, path: &Path, name: &str) -> SpecResult<PathBuf> {
        self.addr.update_rename(path, name).await
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        addr::GitAddr,
        const_vars::SYS_MODEL_INS_ROOT,
        error::SpecResult,
        system::{refs::SysModelSpecRef, spec::SysModelSpec},
        types::{AsyncUpdateable, Localizable},
    };

    #[tokio::test]
    async fn test_sys_running() -> SpecResult<()> {
        let target = "example-sys-x1";
        let spec_ref = SysModelSpecRef::from(
            target,
            GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/spec_example_sys.git")
                .path(target),
        );
        std::fs::create_dir_all(SYS_MODEL_INS_ROOT).assert("yes");
        let root = PathBuf::from(SYS_MODEL_INS_ROOT);
        spec_ref.update_local(&root).await?;
        let spec_path = root.join(target);
        let spec = SysModelSpec::load_from(&spec_path)?;
        spec.update_local().await?;
        spec.localize(None).await?;
        Ok(())
    }
}
