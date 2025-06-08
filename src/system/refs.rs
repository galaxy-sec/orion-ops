use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_getters::Getters;

use crate::{
    addr::AddrType,
    error::SpecResult,
    types::{AsyncUpdateable, UpdateOptions},
};
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
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        self.addr.update_local(path, options).await
    }

    async fn update_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> SpecResult<PathBuf> {
        self.addr.update_rename(path, name, options).await
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssertWithMsg;

    use crate::{
        addr::LocalAddr,
        const_vars::{SYS_MODEL_INS_ROOT, SYS_MODEL_SPC_ROOT},
        error::SpecResult,
        system::{refs::SysModelSpecRef, spec::SysModelSpec},
        tools::test_init,
        types::{AsyncUpdateable, Configable, Localizable, UpdateOptions},
    };

    #[tokio::test]
    async fn test_sys_running() -> SpecResult<()> {
        test_init();
        let target = "example-sys";
        let spec_ref = SysModelSpecRef::from(
            target,
            LocalAddr::from(format!("{}/{}", SYS_MODEL_SPC_ROOT, "example-sys")),
        );
        std::fs::create_dir_all(SYS_MODEL_INS_ROOT).assert("yes");
        let root = PathBuf::from(SYS_MODEL_INS_ROOT);
        spec_ref
            .save_conf(&root.join("sys_model_ref.yml"))
            .assert("save spec_ref");
        spec_ref
            .update_local(&root, &UpdateOptions::for_test())
            .await
            .assert("update_local");
        let spec_path = root.join(target);
        let spec = SysModelSpec::load_from(&spec_path).assert("sysmodel-spec");
        spec.update_local(&UpdateOptions::for_test())
            .await
            .assert("spec.update_local");
        spec.localize(None).await.assert("spec.localize");
        Ok(())
    }
}
