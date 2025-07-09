use crate::{
    error::{SpecReason, ToErr},
    predule::*,
    types::{Localizable, LocalizeOptions, SysUpdateable, ValuePath},
};

use async_trait::async_trait;
use orion_error::{UvsLogicFrom, UvsReason};

use crate::{addr::AddrType, error::SpecResult, types::UnitUpdateable};

use super::spec::SysModelSpec;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysModelSpecRef {
    name: String,
    addr: AddrType,
    #[serde(skip)]
    spec: Option<SysModelSpec>,
}
impl SysModelSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(name: S, addr: A) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
            spec: None,
        }
    }
}

#[async_trait]
impl SysUpdateable<SysModelSpecRef> for SysModelSpecRef {
    async fn update_local(mut self, path: &Path, options: &UpdateOptions) -> SpecResult<Self> {
        let update_v = self.addr.update_local(path, options).await?;
        let spec = SysModelSpec::load_from(update_v.position())?;
        self.spec = Some(spec);
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
