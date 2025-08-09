use crate::error::SysReason;
use crate::predule::*;

use crate::{error::MainResult, module::depend::DependencySet, types::Localizable};

use crate::types::{Accessor, LocalizeOptions, RefUpdateable, ValuePath};
use async_trait::async_trait;
use orion_variate::update::DownloadOptions;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct SysConf {
    test_envs: DependencySet,
}

impl SysConf {
    pub fn new(local_res: DependencySet) -> Self {
        Self {
            test_envs: local_res,
        }
    }
}
#[async_trait]
impl RefUpdateable<()> for SysConf {
    async fn update_local(
        &self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<()> {
        self.test_envs
            .update_local(accessor, path, options)
            .await
            .owe(SysReason::Update.into())
    }
}
#[async_trait]
impl Localizable for SysConf {
    async fn localize(
        &self,
        _val_path: Option<ValuePath>,
        _options: LocalizeOptions,
    ) -> MainResult<()> {
        Ok(())
    }
}
