use crate::{
    error::{SpecError, SpecReason, ToErr},
    predule::*,
    types::{Localizable, LocalizeOptions, SysUpdateable, ValuePath},
};

use async_trait::async_trait;
use contracts::requires;
use orion_error::{UvsLogicFrom, UvsReason};

use crate::{addr::AddrType, error::SpecResult, types::UnitUpdateable};

use super::spec::SysModelSpec;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysModelSpecRef {
    name: String,
    addr: AddrType,
    local: Option<PathBuf>,
    #[serde(skip)]
    spec: Option<SysModelSpec>,
}
impl SysModelSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(name: S, addr: A) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
            local: None,
            spec: None,
        }
    }
    pub fn is_update(&self) -> bool {
        self.local.is_some()
    }

    #[requires(self.local().is_some())]
    pub fn load(mut self) -> SpecResult<Self> {
        let path = self
            .local()
            .clone()
            .ok_or(SpecError::from_logic("spec no local path".into()))?;
        let mut flag = auto_exit_log!(
            info!(
                target : "ops-prj/sys-model",
                "load spec ref to {} success!", path.display()
            ),
            error!(
                target : "ops-prj/sys-model",
                "load spec ref to {} fail!", path.display()
            )
        );
        let spec = SysModelSpec::load_from(&path)?;
        self.spec = Some(spec);
        flag.mark_suc();
        Ok(self)
    }
}

#[async_trait]
impl SysUpdateable<SysModelSpecRef> for SysModelSpecRef {
    async fn update_local(mut self, path: &Path, options: &UpdateOptions) -> SpecResult<Self> {
        let mut flag = auto_exit_log!(
            info!(
                target : "ops-prj/sys-model",
                "update spec ref to {} success!", path.display()
            ),
            error!(
                target : "ops-prj/sys-model",
                "update spec ref to {} fail!", path.display()
            )
        );
        let update_v = self
            .addr
            .update_rename(path, self.name.as_str(), options)
            .await?;
        self.local = Some(update_v.position().clone());
        let spec = SysModelSpec::load_from(update_v.position())?;
        spec.update_local(options).await?;
        self.spec = Some(spec);
        flag.mark_suc();
        Ok(self)
    }
}

#[async_trait]
impl Localizable for SysModelSpecRef {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> SpecResult<()> {
        if let Some(spec) = &self.spec {
            spec.localize(dst_path, options).await?;
            Ok(())
        } else {
            SpecReason::from(UvsReason::from_logic("miss spec from spec-ref".into())).err_result()
        }
    }
}
