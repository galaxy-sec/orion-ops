use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use async_trait::async_trait;
use derive_getters::Getters;
use log::{debug, error, info};
use orion_error::{ErrorOwe, ErrorWith, WithContext};

use crate::{
    action::act::ModWorkflows,
    addr::path_file_name,
    artifact::ArtifactPackage,
    conf::ConfSpec,
    const_vars::{
        ARTIFACT_YML, CONF_SPEC_YML, LOGS_SPEC_YML, RES_SPEC_YML, SETTING_YML, SPEC_DIR, VARS_YML,
    },
    error::{ElementReason, SpecReason, SpecResult, ToErr},
    resource::CaculateResSpec,
    software::LogsSpec,
    types::{
        AsyncUpdateable, Configable, JsonAble, Localizable, LocalizePath, Persistable,
        UpdateOptions,
    },
    vars::{ValueDict, VarCollection},
};

use super::{
    TargetNode,
    locaize::LocalizeTemplate,
    setting::{Setting, TemplateConfig},
};

#[derive(Getters, Clone, Debug)]
pub struct ModTargetSpec {
    target: TargetNode,
    artifact: ArtifactPackage,
    workflow: ModWorkflows,
    conf_spec: ConfSpec,
    logs_spec: LogsSpec,
    res_spec: CaculateResSpec,
    vars: VarCollection,
    local: Option<PathBuf>,
    setting: Option<Setting>,
}

#[async_trait]
impl AsyncUpdateable for ModTargetSpec {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        self.conf_spec.update_local(path, options).await?;
        Ok(path.to_path_buf())
    }
}
impl ModTargetSpec {
    pub fn save_main(&self, root: &Path, name: Option<String>) -> SpecResult<()> {
        let target_path = root.join(name.unwrap_or(self.target().to_string()));
        std::fs::create_dir_all(&target_path)
            .owe_conf()
            .with(format!("path: {}", target_path.display()))?;
        self.workflow.save_to(&target_path, None)?;
        Ok(())
    }
}

#[derive(Getters, Clone, Debug)]
pub struct ModTargetPaths {
    target_root: PathBuf,
    spec_path: PathBuf,
    conf_path: PathBuf,
    logs_path: PathBuf,
    res_path: PathBuf,
    vars_path: PathBuf,
    setting_path: PathBuf,
    artifact_path: PathBuf,
    workflow_path: PathBuf,
}
impl From<&PathBuf> for ModTargetPaths {
    fn from(target_root: &PathBuf) -> Self {
        let spec_path = target_root.join(SPEC_DIR);
        Self {
            target_root: target_root.to_path_buf(),
            conf_path: spec_path.join(CONF_SPEC_YML),
            logs_path: spec_path.join(LOGS_SPEC_YML),
            res_path: spec_path.join(RES_SPEC_YML),
            vars_path: target_root.join(VARS_YML),
            setting_path: target_root.join(SETTING_YML),
            artifact_path: spec_path.join(ARTIFACT_YML),
            workflow_path: target_root.to_path_buf(),
            spec_path,
        }
    }
}

impl Persistable<ModTargetSpec> for ModTargetSpec {
    fn save_to(&self, root: &Path, name: Option<String>) -> SpecResult<()> {
        let target_path = root.join(name.unwrap_or(self.target().to_string()));

        let mut flag = log_flag!(
            info!(target: "spec/mod/target", "save target  success!:{}", target_path.display()),
            error!(target: "spec/mod/target", "save target failed!:{}", target_path.display())
        );
        let paths = ModTargetPaths::from(&target_path);
        std::fs::create_dir_all(paths.spec_path())
            .owe_conf()
            .with(format!("path: {}", paths.spec_path().display()))?;

        if let Some(setting) = &self.setting {
            setting.save_conf(paths.setting_path())?;
        }
        self.workflow.save_to(paths.workflow_path(), None)?;
        self.artifact.save_conf(paths.artifact_path())?;

        self.conf_spec.save_conf(paths.conf_path())?;
        self.logs_spec.save_conf(paths.logs_path())?;

        self.res_spec.save_conf(paths.res_path())?;
        self.vars.save_conf(paths.vars_path())?;
        flag.flag_suc();
        Ok(())
    }

    fn load_from(target_root: &Path) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load target mod spec");

        let mut flag = log_flag!(
            info!(target: "spec/mod/target", "load target  success!:{}", target_root.display()),
            error!(target: "spec/mod/target", "load target failed!:{}", target_root.display())
        );
        let paths = ModTargetPaths::from(&target_root.to_path_buf());
        let target = TargetNode::from_str(path_file_name(target_root)?.as_str())
            .owe_res()
            .with(&ctx)?;
        let actions = ModWorkflows::load_from(paths.workflow_path()).with(&ctx)?;

        let setting = if paths.setting_path().exists() {
            Some(Setting::from_conf(paths.setting_path())?)
        } else {
            None
        };
        ctx.with_path("artifact", paths.artifact_path());
        let artifact = ArtifactPackage::from_conf(paths.artifact_path()).with(&ctx)?;

        ctx.with_path("conf_spec", paths.conf_path());
        let conf_spec = ConfSpec::from_conf(paths.conf_path()).with(&ctx)?;
        ctx.with_path("logs_spec", paths.logs_path());
        let logs_spec = LogsSpec::from_conf(paths.logs_path()).with(&ctx)?;
        ctx.with_path("res_spec", paths.res_path());
        let res_spec = CaculateResSpec::from_conf(paths.res_path()).with(&ctx)?;
        ctx.with_path("vars", paths.vars_path());
        let vars = VarCollection::from_conf(paths.vars_path()).with(&ctx)?;

        flag.flag_suc();
        Ok(Self {
            target,
            artifact,
            workflow: actions,
            conf_spec,
            logs_spec,
            res_spec,
            local: Some(target_root.to_path_buf()),
            vars,
            setting,
        })
    }
}
impl ModTargetSpec {
    pub fn init(
        target: TargetNode,
        artifact: ArtifactPackage,
        actions: ModWorkflows,
        conf_spec: ConfSpec,
        res_spec: CaculateResSpec,
        vars: VarCollection,
        setting: Option<Setting>,
    ) -> Self {
        Self {
            target,
            workflow: actions,
            artifact,
            conf_spec,
            logs_spec: LogsSpec::tpl_init(),
            res_spec,
            local: None,
            vars,
            setting,
        }
    }
}

#[async_trait]
impl Localizable for ModTargetSpec {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()> {
        let mut ctx = WithContext::want("modul localize");
        let local = self.local.clone().ok_or(
            SpecReason::from(ElementReason::Miss("local-path".into()))
                .to_err()
                .with(&ctx),
        )?;
        let tpl = local.join(crate::const_vars::SPEC_DIR);
        let localize_path = dst_path.unwrap_or(LocalizePath::new(
            local.join(crate::const_vars::LOCAL_DIR),
            local.clone(),
        ));

        let value_path = localize_path.value().join(crate::const_vars::VALUE_JSON);
        let used_path = localize_path.value().join(crate::const_vars::USED_JSON);
        let local_path = localize_path.local();
        debug!( target:"spec/mod/target", "localize mod-target begin: {}" ,local_path.display() );
        if local_path.exists() {
            std::fs::remove_dir_all(local_path).owe_res()?;
        }
        std::fs::create_dir_all(local_path).owe_res()?;

        ctx.with_path("dst", local_path);
        self.update_local(&tpl, &UpdateOptions::default()).await?;
        if !value_path.exists() {
            value_path.parent().map(std::fs::create_dir_all);
            let vars_dict = self.vars.value_dict();
            vars_dict.save_json(&value_path)?;
        }
        if let Some(global) = localize_path.global() {
            let mut used = ValueDict::from_json(global)?;
            let sec = ValueDict::from_json(&value_path)?;
            used.merge(&sec);
            used.save_json(&used_path)?;
        } else {
            let used = ValueDict::from_json(&value_path)?;
            used.save_json(&used_path)?;
        }
        let tpl_path_opt = self
            .setting
            .as_ref()
            .and_then(|x| x.localize().clone())
            .and_then(|x| x.templatize_path().clone())
            .map(|x| x.export_paths(&local));

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
        localizer
            .render_path(&tpl, local_path, &used_path, &tpl_path)
            .with(&ctx)?;
        info!( target:"spec/mod/target", "localize mod-target success!: {}" ,local_path.display() );
        Ok(())
    }
}
