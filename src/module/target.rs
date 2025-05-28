use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};

use crate::{
    action::act::ModWorkflows,
    addr::path_file_name,
    artifact::ArtifactPackage,
    conf::ConfSpec,
    const_vars::SPEC_DIR,
    error::{SpecReason, SpecResult, ToErr},
    resource::CaculateResSpec,
    software::LogsSpec,
    tpl::{TPlEngineType, TplRender},
    types::{AsyncUpdateable, Configable, JsonAble, Localizable, LocalizePath, Persistable},
    vars::{ValueDict, VarCollection},
};

use super::TargetNode;

#[derive(Getters, Clone, Debug)]
pub struct ModTargetSpec {
    target: TargetNode,
    artifact: ArtifactPackage,
    actions: ModWorkflows,
    conf_spec: ConfSpec,
    logs_spec: LogsSpec,
    res_spec: CaculateResSpec,
    vars: VarCollection,
    local: Option<PathBuf>,
}

#[async_trait]
impl AsyncUpdateable for ModTargetSpec {
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        self.conf_spec.update_local(path).await?;
        Ok(path.to_path_buf())
    }
}

impl Persistable<ModTargetSpec> for ModTargetSpec {
    fn save_to(&self, root: &Path) -> SpecResult<()> {
        let target_path = root.join(self.target().to_string());
        let spec_path = root.join(self.target().to_string()).join(SPEC_DIR);
        std::fs::create_dir_all(&spec_path)
            .owe_conf()
            .with(format!("path: {}", spec_path.display()))?;
        self.actions.save_to(&target_path)?;
        let artifact_path = spec_path.join(crate::const_vars::ARTIFACT_YML);
        self.artifact.save_conf(&artifact_path)?;

        let conf_path = spec_path.join(crate::const_vars::CONF_SPEC_YML);
        self.conf_spec.save_conf(&conf_path)?;
        let logs_path = spec_path.join(crate::const_vars::LOGS_SPEC_YML);
        self.logs_spec.save_conf(&logs_path)?;

        let res_path = spec_path.join("res_spec.yml");
        self.res_spec.save_conf(&res_path)?;
        let vars_path = spec_path.join("vars.yml");
        self.vars.save_conf(&vars_path)?;
        Ok(())
    }

    fn load_from(root_path: &Path) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load target mod spec");

        let target = TargetNode::from_str(path_file_name(root_path)?.as_str())
            .owe_res()
            .with(&ctx)?;
        let actions = ModWorkflows::load_from(root_path).with(&ctx)?;
        let target_path = root_path.join(SPEC_DIR);
        let artifact_path = target_path.join("artifact.yml");
        ctx.with_path("artifact", &artifact_path);
        let artifact = ArtifactPackage::from_conf(&artifact_path).with(&ctx)?;

        let spec_path = target_path.join("conf_spec.yml");
        ctx.with_path("conf_spec", &spec_path);
        let conf_spec = ConfSpec::from_conf(&spec_path).with(&ctx)?;
        let logs_path = target_path.join(crate::const_vars::LOGS_SPEC_YML);
        ctx.with_path("logs_spec", &logs_path);
        let logs_spec = LogsSpec::from_conf(&logs_path).with(&ctx)?;
        let res_path = target_path.join(crate::const_vars::RES_SPEC_YML);
        ctx.with_path("res_spec", &logs_path);
        let res_spec = CaculateResSpec::from_conf(&res_path).with(&ctx)?;
        let vars_path = target_path.join(crate::const_vars::VARS_YML);
        ctx.with_path("vars", &vars_path);
        let vars = VarCollection::from_conf(&vars_path).with(&ctx)?;

        Ok(Self {
            target,
            artifact,
            actions,
            conf_spec,
            logs_spec,
            res_spec,
            local: Some(root_path.to_path_buf()),
            vars,
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
    ) -> Self {
        Self {
            target,
            actions,
            artifact,
            conf_spec,
            logs_spec: LogsSpec::tpl_init(),
            res_spec,
            local: None,
            vars,
        }
    }
}

#[async_trait]
impl Localizable for ModTargetSpec {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()> {
        let mut ctx = WithContext::want("modul localize");
        let local = self
            .local
            .clone()
            .ok_or(SpecReason::Miss("local-path".into()).to_err().with(&ctx))?;
        let tpl = local.join(crate::const_vars::SPEC_DIR);
        let localize_path = dst_path.unwrap_or(LocalizePath::new(
            local.join(crate::const_vars::LOCAL_DIR),
            local,
        ));

        let value_path = localize_path.value().join(crate::const_vars::VALUE_JSON);
        let used_path = localize_path.value().join(crate::const_vars::USED_JSON);
        let local_path = localize_path.local();

        ctx.with_path("dst", local_path);
        self.update_local(&tpl).await?;
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

        TplRender::render_path(TPlEngineType::Handlebars, &tpl, local_path, &used_path)
            .with(&ctx)?;
        Ok(())
    }
}
