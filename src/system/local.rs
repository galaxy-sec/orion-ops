use crate::{
    error::{MainError, MainReason, SysReason, ToErr},
    module::{
        localize::LocalizeTemplate,
        setting::{Setting, TemplateConfig},
    },
    predule::*,
    types::{Accessor, InsUpdateable, Localizable, LocalizeOptions, RefUpdateable, ValuePath},
};

use async_trait::async_trait;
use chrono::Local;
use derive_more::Deref;
use orion_error::{UvsLogicFrom, UvsReason};
use orion_infra::auto_exit_log;
use orion_variate::{addr::Address, types::ResourceDownloader, update::DownloadOptions};

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct LocalizePath {
    src: PathBuf,
    dst: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    setting: Option<Setting>,
}
#[derive(Getters, Clone, Debug, Default, Serialize, Deserialize, Deref)]
#[serde(transparent)]
pub struct LocalizeSet {
    items: Vec<LocalizePath>,
}

#[async_trait]
impl RefUpdateable<()> for LocalizeSet {
    async fn update_local(
        &self,
        accessor: Accessor,
        _path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<()> {
        todo!();
    }
}
#[async_trait]
impl Localizable for LocalizeSet {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> MainResult<()> {
        todo!();
    }
}

#[async_trait]
impl Localizable for LocalizePath {
    async fn localize(
        &self,
        val_path: Option<ValuePath>,
        _options: LocalizeOptions,
    ) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(target : "/sys/localize", "sys-path localize {} success!", self.dst.display()),
            error!(target: "/sys/localize", "sys-path localize {} fail!",
                self.dst.display())
        );
        let mut ctx = WithContext::want("sys-path localize");
        ctx.with_path("dst", &self.dst);
        ctx.with_path("src", &self.src);
        let tpl_path_opt = self
            .setting
            .as_ref()
            .and_then(|x| x.localize().clone())
            .and_then(|x| x.templatize_path().clone())
            .map(|x| x.export_paths(&self.dst));

        let tpl_path = tpl_path_opt.unwrap_or_default();
        let tpl_custom = self
            .setting
            .as_ref()
            .and_then(|x| x.localize().clone())
            .and_then(|x| x.templatize_cust().clone())
            .map(TemplateConfig::from);

        let localizer = if let Some(cust) = tpl_custom {
            LocalizeTemplate::new(cust)
        } else {
            LocalizeTemplate::default()
        };
        let value_file =
            val_path.ok_or_else(|| MainError::from_logic("miss sys value file ".into()))?;
        if !value_file.path().exists() {
            return Err(MainError::from_logic(format!(
                "sys value file not exists :{} ",
                value_file.path().display()
            )));
        }
        localizer
            .render_path(&self.src, &self.dst, value_file.path(), &tpl_path)
            .with(&ctx)?;
        flag.mark_suc();
        Ok(())
    }
}
#[cfg(test)]
mod tests {}
