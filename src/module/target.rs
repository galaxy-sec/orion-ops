use crate::{
    const_vars::{
        DEFAULT_VALUE_FILE, LOCAL_DIR, SAMPLE_VALUE_FILE, USED_JSON, USED_READABLE_FILE,
        USER_VALUE_FILE, VALUE_DIR,
    },
    predule::*,
    tools::ensure_path,
};
use std::{fs::read_to_string, str::FromStr};

use async_trait::async_trait;

use super::{
    TargetNode,
    depend::DependencySet,
    localize::LocalizeTemplate,
    setting::{Setting, TemplateConfig},
};
use crate::tools::make_clean_path;
use crate::types::LocalizeOptions;
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
        AsyncUpdateable, Configable, JsonAble, Localizable, Persistable, ValueConfable, ValuePath,
    },
    vars::{OriginDict, ValueDict, VarCollection},
    workflow::{act::ModWorkflows, prj::GxlProject},
};

#[derive(Getters, Clone, Debug, Serialize)]
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

    fn build_used_value(
        &self,
        options: LocalizeOptions,
        value_paths: &TargetValuePaths,
    ) -> Result<OriginDict, StructError<SpecReason>> {
        let mut used = OriginDict::from(options.global_value().clone());
        used.set_source("global");
        if value_paths.user_value_file().exists() && options.use_custom_value() {
            let mut user_dict = OriginDict::from(ValueDict::eval_from_file(
                &used.export_dict(),
                value_paths.user_value_file(),
            )?);
            user_dict.set_source("mod-cust");
            used.merge(&user_dict);
        }
        let mut default_dict =
            OriginDict::from(self.vars.value_dict().env_eval(&used.export_dict()));
        default_dict.set_source("mod-default");
        used.merge(&default_dict);
        Ok(used)
    }

    fn crate_sample_value_file(
        &self,
        value_paths: &TargetValuePaths,
    ) -> Result<(), StructError<SpecReason>> {
        if !(value_paths.sample_value_file().exists() || value_paths.user_value_file().exists()) {
            value_paths
                .sample_value_file()
                .parent()
                .map(std::fs::create_dir_all);
            let vars_dict = self.vars.value_dict();
            vars_dict.save_valconf(value_paths.sample_value_file())?;
            info!( target:"mod/target", "crate  value.yml at : {}" ,value_paths.sample_value_file().display() );
        }
        Ok(())
    }
}

#[async_trait]
impl AsyncUpdateable for ModTargetSpec {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        //self.conf_spec.update_local(path, options).await?;
        self.depends.update(options).await?;
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

#[derive(Getters, Clone, Debug)]
pub struct TargetValuePaths {
    used_readable: PathBuf,
    default_value_file: PathBuf,
    user_value_file: PathBuf,
    sample_value_file: PathBuf,
    used_json_path: PathBuf,
}
impl From<&PathBuf> for TargetValuePaths {
    fn from(value_root: &PathBuf) -> Self {
        Self {
            used_readable: value_root.join(USED_READABLE_FILE),
            default_value_file: value_root.join(DEFAULT_VALUE_FILE),
            user_value_file: value_root.join(USER_VALUE_FILE),
            sample_value_file: value_root.join(SAMPLE_VALUE_FILE),
            used_json_path: value_root.join(crate::const_vars::USED_JSON),
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
    pub fn get_local_values(&self, parent: ValuePath) -> SpecResult<Option<String>> {
        let value_paths = TargetValuePaths::from(parent.path());
        if value_paths.used_readable().exists() {
            let data = read_to_string(value_paths.used_readable()).owe_sys()?;
            return Ok(Some(data));
        }
        Ok(None)
    }
}

#[async_trait]
impl Localizable for ModTargetSpec {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
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
        let localize_path = dst_path.unwrap_or(ValuePath::new(local.join(VALUE_DIR)));

        let value_root = localize_path.path(); //.join(VALUE_DIR);
        let value_paths = TargetValuePaths::from(value_root);
        let used_value_path = ensure_path(local.join(VALUE_DIR))?;
        let used_value_file = used_value_path.join(USED_JSON);
        let local_path = local.join(LOCAL_DIR);
        debug!( target:"spec/mod/target", "localize mod-target begin: {}" ,local_path.display() );
        make_clean_path(&local_path)?;
        ctx.with_path("dst", &local_path);
        self.crate_sample_value_file(&value_paths)?;
        debug!(target : "/mod/target/loc", "value export");
        let used = self.build_used_value(options, &value_paths)?;
        used.export_origin()
            //.env_eval()
            .save_valconf(value_paths.used_readable())?;
        used.export_value().save_json(&used_value_file)?;

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
            .render_path(&tpl, &local_path, &used_value_file, &tpl_path)
            .with(&ctx)?;
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
        vars::{OriginValue, ValueType, VarType},
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
        loaded
            .localize(None, LocalizeOptions::for_test())
            .await
            .assert();
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
        loaded
            .localize(None, LocalizeOptions::for_test())
            .await
            .assert();
        Ok(())
    }

    fn build_spec(vars: VarCollection) -> ModTargetSpec {
        let spec = ModTargetSpec::init(
            TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
            ArtifactPackage::default(),
            ModWorkflows::default(),
            GxlProject::default(),
            CaculateResSpec::new(2, 4),
            vars,
            None,
        );
        spec
    }

    #[test]
    fn test_build_used_value_with_default_only() {
        test_init();
        let vars = VarCollection::define(vec![VarType::from(("TEST_KEY", "default_value"))]);
        let spec = build_spec(vars);
        let options = LocalizeOptions::new(ValueDict::new(), false);
        let temp_dir = tempfile::tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = spec.build_used_value(options, &value_paths).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("default_value").with_origin("mod-default"))
        );
    }

    #[test]
    fn test_build_used_value_with_global_value() {
        test_init();
        let mut global_dict = ValueDict::new();
        global_dict.insert("TEST_KEY".to_string(), ValueType::from("global_value"));
        let vars = VarCollection::define(vec![VarType::from(("TEST_KEY", "default_value"))]);
        let spec = build_spec(vars);
        let options = LocalizeOptions::new(global_dict, false);
        let temp_dir = tempfile::tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = spec.build_used_value(options, &value_paths).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            //Some(&Value::String("global_value".to_string()))
            Some(&OriginValue::from("global_value").with_origin("global"))
        );
    }

    #[test]
    fn test_build_used_value_with_user_value() {
        test_init();
        let temp_dir = tempfile::tempdir().unwrap();
        let user_value_path = temp_dir.path().join(USER_VALUE_FILE);
        std::fs::write(&user_value_path, "TEST_KEY: user_value").unwrap();

        let vars = VarCollection::define(vec![VarType::from(("TEST_KEY", "default_value"))]);
        let spec = build_spec(vars);
        let options = LocalizeOptions::new(ValueDict::new(), true);
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = spec.build_used_value(options, &value_paths).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("user_value").with_origin("mod-cust"))
        );
    }

    #[test]
    fn test_build_used_value_merge_precedence() {
        test_init();
        let temp_dir = tempfile::tempdir().unwrap();
        let cust_value_path = temp_dir.path().join(USER_VALUE_FILE);
        std::fs::write(
            &cust_value_path,
            "TEST_KEY: user_value\nUSER_ONLY: user_only",
        )
        .unwrap();

        let mut global_dict = ValueDict::new();
        global_dict.insert("TEST_KEY".to_string(), ValueType::from("global_value"));
        global_dict.insert("GLOBAL_ONLY".to_string(), ValueType::from("global_only"));

        let vars = VarCollection::define(vec![
            VarType::from(("TEST_KEY", "default_value")),
            VarType::from(("DEFAULT_ONLY", "default_only")),
        ]);
        let spec = build_spec(vars);
        let options = LocalizeOptions::new(global_dict, true);
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = spec.build_used_value(options, &value_paths).unwrap();
        // 验证优先级: global > cust  > default
        assert_eq!(
            result.get("TEST_KEY"),
            //Some(&Value::String("user_value".to_string()))
            Some(&OriginValue::from("global_value").with_origin("global"))
        );
        // 验证各层特有键都存在
        assert_eq!(
            result.get("GLOBAL_ONLY"),
            //Some(&Value::String("global_only".to_string()))
            Some(&OriginValue::from("global_only").with_origin("global"))
        );
        assert_eq!(
            result.get("USER_ONLY"),
            //Some(&Value::String("user_only".to_string()))
            Some(&OriginValue::from("user_only").with_origin("mod-cust"))
        );
        assert_eq!(
            result.get("DEFAULT_ONLY"),
            //Some(&Value::String("default_only".to_string()))
            Some(&OriginValue::from("default_only").with_origin("mod-default"))
        );
    }
}
