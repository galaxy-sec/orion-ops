use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{const_vars::VALUE_DIR, vars::EnvEvalable};
use async_trait::async_trait;
use derive_getters::Getters;
use log::{debug, error, info};
use orion_error::{ErrorOwe, ErrorWith, WithContext};

use crate::{
    addr::path_file_name,
    artifact::ArtifactPackage,
    const_vars::{
        ARTIFACT_YML, CONF_SPEC_YML, DEPENDS_YML, LOGS_SPEC_YML, RES_SPEC_YML, SETTING_YML,
        SPEC_DIR, VARS_YML,
    },
    error::{ElementReason, SpecReason, SpecResult, ToErr},
    resource::CaculateResSpec,
    software::LogsSpec,
    tools::get_sub_dirs,
    types::{
        AsyncUpdateable, Configable, JsonAble, Localizable, LocalizePath, Persistable,
        UpdateOptions, ValueConfable,
    },
    vars::{OriginDict, ValueDict, VarCollection},
    workflow::{act::ModWorkflows, prj::GxlProject},
};

use super::{
    TargetNode,
    depend::DependencySet,
    localize::LocalizeTemplate,
    setting::{Setting, TemplateConfig},
};

#[derive(Getters, Clone, Debug)]
pub struct ModTargetSpec {
    target: TargetNode,
    artifact: ArtifactPackage,
    workflow: ModWorkflows,
    gxl_prj: GxlProject,
    logs_spec: LogsSpec,
    res_spec: CaculateResSpec,
    vars: VarCollection,
    local: Option<PathBuf>,
    setting: Option<Setting>,
    depends: DependencySet,
}

impl ModTargetSpec {
    pub fn with_depends(mut self, depends: DependencySet) -> Self {
        self.depends = depends;
        self
    }
}

#[async_trait]
impl AsyncUpdateable for ModTargetSpec {
    async fn update_local(&self, path: &Path, _options: &UpdateOptions) -> SpecResult<PathBuf> {
        //self.conf_spec.update_local(path, options).await?;
        self.depends.update().await?;
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

    pub fn clean_other(root: &Path, node: &TargetNode) -> SpecResult<()> {
        let subs = get_sub_dirs(root)?;
        for sub in subs {
            if !sub.ends_with(node.to_string().as_str()) {
                Self::clean_path(&sub)?;
            }
        }
        Ok(())
    }
    fn clean_path(path: &Path) -> SpecResult<()> {
        if path.exists() {
            std::fs::remove_dir_all(path).owe_res().with(path)?;
        }
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
    depends_path: PathBuf,
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
            depends_path: spec_path.join(DEPENDS_YML),
            workflow_path: target_root.to_path_buf(),
            spec_path,
        }
    }
}

impl Persistable<ModTargetSpec> for ModTargetSpec {
    fn save_to(&self, root: &Path, name: Option<String>) -> SpecResult<()> {
        let target_path = root.join(name.unwrap_or(self.target().to_string()));

        let mut flag = log_guard!(
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

        self.depends.save_conf(paths.depends_path())?;
        //self.conf_spec.save_conf(paths.conf_path())?;
        self.logs_spec.save_conf(paths.logs_path())?;

        self.res_spec.save_conf(paths.res_path())?;
        self.vars.save_conf(paths.vars_path())?;
        self.gxl_prj.save_to(&paths.target_root, None)?;
        flag.flag_suc();
        Ok(())
    }

    fn load_from(target_root: &Path) -> SpecResult<Self> {
        let mut ctx = WithContext::want("load target mod spec");

        let mut flag = log_guard!(
            info!(target: "spec/mod/target", "load target  success!:{}", target_root.display()),
            error!(target: "spec/mod/target", "load target failed!:{}", target_root.display())
        );
        let paths = ModTargetPaths::from(&target_root.to_path_buf());
        ctx.with_path("root", target_root);
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

        //ctx.with_path("conf_spec", paths.conf_path());
        //let conf_spec = ConfSpec::from_conf(paths.conf_path()).with(&ctx)?;
        ctx.with_path("logs_spec", paths.logs_path());
        let logs_spec = LogsSpec::from_conf(paths.logs_path()).with(&ctx)?;

        ctx.with_path("depends", paths.depends_path());
        let depends = DependencySet::from_conf(paths.depends_path()).with(&ctx)?;
        ctx.with_path("res_spec", paths.res_path());
        let res_spec = CaculateResSpec::from_conf(paths.res_path()).with(&ctx)?;
        ctx.with_path("vars", paths.vars_path());
        let vars = VarCollection::from_conf(paths.vars_path()).with(&ctx)?;

        let gxl_prj = GxlProject::load_from(paths.target_root()).with(&ctx)?;
        flag.flag_suc();
        Ok(Self {
            target,
            artifact,
            workflow: actions,
            //conf_spec,
            logs_spec,
            res_spec,
            local: Some(target_root.to_path_buf()),
            vars,
            setting,
            depends,
            gxl_prj,
        })
    }
}
impl ModTargetSpec {
    pub fn init(
        target: TargetNode,
        artifact: ArtifactPackage,
        workflow: ModWorkflows,
        gxl_prj: GxlProject,
        //conf_spec: ConfSpec,
        res_spec: CaculateResSpec,
        vars: VarCollection,
        setting: Option<Setting>,
    ) -> Self {
        Self {
            target,
            workflow,
            gxl_prj,
            artifact,
            //conf_spec,
            logs_spec: LogsSpec::tpl_init(),
            res_spec,
            local: None,
            vars,
            setting,
            depends: DependencySet::default(),
        }
    }
}

#[async_trait]
impl Localizable for ModTargetSpec {
    async fn localize(
        &self,
        dst_path: Option<LocalizePath>,
        global_value: Option<PathBuf>,
    ) -> SpecResult<()> {
        let mut flag = log_guard!(
            info!(target : "/mod/target", "mod-target localize {} success!", self.target()),
            error!(target: "/mod/target", "mod-target localize {} fail!",
                self.local.clone().unwrap_or(PathBuf::from("unknow")).display())
        );
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

        let value_root = localize_path.value().join(VALUE_DIR);
        let value_path = value_root.join(crate::const_vars::VALUE_FILE);
        let used_readable = value_root.join(crate::const_vars::USED_READABLE_FILE);
        let used_json_path = value_root.join(crate::const_vars::USED_JSON);
        let local_path = localize_path.local();
        debug!( target:"spec/mod/target", "localize mod-target begin: {}" ,local_path.display() );
        if local_path.exists() {
            std::fs::remove_dir_all(local_path).owe_res()?;
        }
        std::fs::create_dir_all(local_path).owe_res()?;

        ctx.with_path("dst", local_path);
        if !value_path.exists() {
            value_path.parent().map(std::fs::create_dir_all);
            let vars_dict = self.vars.value_dict();
            vars_dict.save_valconf(&value_path)?;
        }
        debug!(target : "/mod/target/loc", "value export");
        if let Some(global) = global_value {
            let mut used = OriginDict::from(ValueDict::from_valconf(&global)?);
            used.set_source("global");
            let mut cur_mod = OriginDict::from(ValueDict::from_valconf(&value_path)?);
            cur_mod.set_source("mod");
            used.merge(&cur_mod);
            used.export_origin()
                .env_eval()
                .save_valconf(&used_readable)?;
            used.export_value().env_eval().save_json(&used_json_path)?;
        } else {
            //let used = ValueDict::from_valconf(&value_path)?;
            let used = OriginDict::from(ValueDict::from_valconf(&value_path)?);

            used.export_origin()
                .env_eval()
                .save_valconf(&used_readable)?;
            used.export_value().env_eval().save_json(&used_json_path)?;
        }
        debug!(target : "/mod/target/loc", "update_local suc");
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
            .render_path(&tpl, local_path, &used_json_path, &tpl_path)
            .with(&ctx)?;
        //info!( target:"spec/mod/target", "localize mod-target success!: {}" ,local_path.display() );
        flag.flag_suc();
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use orion_error::TestAssert;

    use crate::{
        addr::HttpAddr,
        artifact::Artifact,
        const_vars::TARGET_SPC_ROOT,
        error::SpecResult,
        module::{
            CpuArch, OsCPE, RunSPC,
            init::{ModIniter, ModPrjIniter},
        },
        tools::{make_clean_path, test_init},
        vars::VarType,
    };

    use super::*;

    pub fn make_mod_k8s_4test() -> SpecResult<ModTargetSpec> {
        let name = "postgresql";
        let k8s = ModTargetSpec::init(
            TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
            ArtifactPackage::from(vec![Artifact::new(
                name,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz",
                ),
                "postgresql-17.4.tar.gz",
            )]),
            ModWorkflows::mod_k8s_tpl_init(),
            GxlProject::spec_k8s_tpl(),
            //conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
            Some(Setting::example()),
        )
        .with_depends(DependencySet::for_test());
        Ok(k8s)
    }

    pub fn make_mod_host_4test() -> SpecResult<ModTargetSpec> {
        let name = "postgresql";
        let host = ModTargetSpec::init(
            TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
            ArtifactPackage::from(vec![Artifact::new(
                name,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz",
                ),
                "postgresql-17.4.tar.gz",
            )]),
            ModWorkflows::mod_host_tpl_init(),
            GxlProject::spec_host_tpl(),
            //conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
            Some(Setting::example()),
        )
        .with_depends(DependencySet::for_test());
        Ok(host)
    }

    #[tokio::test]
    async fn build_target_k8s() -> SpecResult<()> {
        test_init();
        let spec = make_mod_k8s_4test().assert();
        let spec_path = PathBuf::from(TARGET_SPC_ROOT).join(spec.target().to_string());
        make_clean_path(&spec_path)?;
        spec.save_to(&PathBuf::from(TARGET_SPC_ROOT), None).assert();
        let loaded = ModTargetSpec::load_from(
            &PathBuf::from(TARGET_SPC_ROOT).join(spec.target().to_string()),
        )
        .assert();
        loaded.localize(None, None).await.assert();
        Ok(())
    }

    #[tokio::test]
    async fn build_target_host() -> SpecResult<()> {
        test_init();
        let spec = make_mod_host_4test().assert();
        let spec_path = PathBuf::from(TARGET_SPC_ROOT).join(spec.target().to_string());
        make_clean_path(&spec_path)?;
        spec.save_to(&PathBuf::from(TARGET_SPC_ROOT), None).assert();
        let loaded = ModTargetSpec::load_from(
            &PathBuf::from(TARGET_SPC_ROOT).join(spec.target().to_string()),
        )
        .assert();
        loaded.localize(None, None).await.assert();
        Ok(())
    }
}
