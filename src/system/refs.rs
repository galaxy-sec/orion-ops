use crate::{
    error::{MainReason, SysReason, ToErr},
    predule::*,
    types::{Accessor, InsUpdateable, Localizable, LocalizeOptions, RefUpdateable, ValuePath},
};

use async_trait::async_trait;
use orion_error::{UvsLogicFrom, UvsReason};
use orion_infra::auto_exit_log;
use orion_variate::{addr::Address, types::ResourceDownloader, update::DownloadOptions};

use crate::error::MainResult;

use super::spec::SysModelSpec;

fn convert_syspec_addr(origin: Address) -> Address {
    match origin {
        Address::Git(git_addr) => {
            if git_addr.path().is_none() {
                Address::from(git_addr.with_path("sys"))
            } else {
                Address::from(git_addr)
            }
        }
        Address::Http(_) => origin,
        Address::Local(_) => origin,
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysModelSpecRef {
    name: String,
    addr: Address,
    #[serde(skip)]
    spec: Option<SysModelSpec>,
}
impl SysModelSpecRef {
    pub fn from<S: Into<String>, A: Into<Address>>(name: S, addr: A) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
            spec: None,
        }
    }
    pub fn is_update(&self, path: &Path) -> bool {
        path.join(self.name()).exists()
    }

    pub fn load(mut self, path: &Path) -> MainResult<Self> {
        let path = path.join(self.name());
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
impl InsUpdateable<SysModelSpecRef> for SysModelSpecRef {
    async fn update_local(
        mut self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<Self> {
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
        let spec_addr = convert_syspec_addr(self.addr.clone());
        let update_v = accessor
            .download_rename(&spec_addr, path, self.name.as_str(), options)
            .await
            .owe(SysReason::Update.into())?;
        let spec = SysModelSpec::load_from(update_v.position())?;
        spec.update_local(accessor, path, options).await?;
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
    ) -> MainResult<()> {
        if let Some(spec) = &self.spec {
            spec.localize(dst_path, options).await?;
            Ok(())
        } else {
            MainReason::from(UvsReason::from_logic("miss spec from spec-ref".into())).err_result()
        }
    }
}
