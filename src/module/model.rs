use super::prelude::*;
use crate::{
    artifact::ArtifactPackage,
    const_vars::{
        DEFAULT_VALUE_FILE, LOCAL_DIR, SAMPLE_VALUE_FILE, USED_JSON, USED_READABLE_FILE,
        USER_VALUE_FILE, VALUE_DIR,
    },
    predule::*,
    types::{Localizable, RefUpdateable, ValuePath},
};
use std::{fs::read_to_string, str::FromStr};

use super::{
    ModelSTD,
    depend::DependencySet,
    localize::LocalizeTemplate,
    setting::{Setting, TemplateConfig},
};

#[derive(Getters, Clone, Debug, Serialize)]
pub struct ModModelSpec {
    model: ModelSTD,
    artifact: ArtifactPackage,
    workflow: ModWorkflows,
    gxl_prj: GxlProject,
    vars: VarCollection,
    local: Option<PathBuf>,
    setting: Option<Setting>,
    depends: DependencySet,
}

impl ModModelSpec {
    pub fn with_depends(mut self, depends: DependencySet) -> Self {
        self.depends = depends;
        self
    }

    fn build_used_value(
        &self,
        options: LocalizeOptions,
        value_paths: &TargetValuePaths,
    ) -> Result<OriginDict, StructError<MainReason>> {
        crate::project::mix_used_value(options, value_paths, &self.vars)
    }

    fn crate_sample_value_file(
        &self,
        value_paths: &TargetValuePaths,
    ) -> Result<(), StructError<MainReason>> {
        if !(value_paths.sample_value_file().exists() || value_paths.user_value_file().exists()) {
            value_paths
                .sample_value_file()
                .parent()
                .map(std::fs::create_dir_all);
            let vars_dict = self.vars.value_dict();
            vars_dict
                .save_valconf(value_paths.sample_value_file())
                .owe_res()?;
            info!( target:"mod/target", "crate  value.yml at : {}" ,value_paths.sample_value_file().display() );
        }
        Ok(())
    }
}

#[async_trait]
impl RefUpdateable<UpdateUnit> for ModModelSpec {
    async fn update_local(
        &self,
        accessor: Accessor,
        path: &Path,
        options: &DownloadOptions,
    ) -> MainResult<UpdateUnit> {
        //self.conf_spec.update_local(path, options).await?;
        self.depends.update_local(accessor, path, options).await?;
        Ok(UpdateUnit::new(path.to_path_buf(), self.vars.clone()))
    }
}
impl ModModelSpec {
    pub fn save_main(&self, root: &Path, name: Option<String>) -> MainResult<()> {
        let target_path = root.join(name.unwrap_or(self.model().to_string()));
        std::fs::create_dir_all(&target_path)
            .owe_conf()
            .with(format!("path: {}", target_path.display()))?;
        self.workflow.save_to(&target_path, None).owe_logic()?;
        Ok(())
    }

    pub fn clean_other(root: &Path, node: &ModelSTD) -> MainResult<()> {
        let subs = get_sub_dirs(root).owe_logic()?;
        for sub in subs {
            if !sub.ends_with(node.to_string().as_str()) {
                Self::clean_path(&sub)?;
            }
        }
        Ok(())
    }
    fn clean_path(path: &Path) -> MainResult<()> {
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

impl Persistable<ModModelSpec> for ModModelSpec {
    fn save_to(&self, root: &Path, name: Option<String>) -> SerdeResult<()> {
        let target_path = root.join(name.unwrap_or(self.model().to_string()));

        let mut flag = auto_exit_log!(
            info!(target: "spec/mod/target", "save target  success!:{}", target_path.display()),
            error!(target: "spec/mod/target", "save target failed!:{}", target_path.display())
        );
        let paths = ModTargetPaths::from(&target_path);
        std::fs::create_dir_all(paths.spec_path())
            .owe_conf()
            .with(format!("path: {}", paths.spec_path().display()))?;

        if let Some(setting) = &self.setting {
            setting.save_conf(paths.setting_path()).owe_logic()?;
        }
        self.workflow.save_to(paths.workflow_path(), None)?;
        self.artifact.save_conf(paths.artifact_path()).owe_logic()?;

        self.depends.save_conf(paths.depends_path()).owe_logic()?;
        self.vars.save_conf(paths.vars_path()).owe_logic()?;
        self.gxl_prj.save_to(&paths.target_root, None)?;
        flag.mark_suc();
        Ok(())
    }

    fn load_from(target_root: &Path) -> SerdeResult<Self> {
        let mut ctx = WithContext::want("load target mod spec");

        let mut flag = auto_exit_log!(
            info!(target: "spec/mod/target", "load target  success!:{}", target_root.display()),
            error!(target: "spec/mod/target", "load target failed!:{}", target_root.display())
        );
        let paths = ModTargetPaths::from(&target_root.to_path_buf());
        ctx.with_path("root", target_root);
        let target = ModelSTD::from_str(path_file_name(target_root).owe_logic()?.as_str())
            .owe_res()
            .with(&ctx)?;
        let actions = ModWorkflows::load_from(paths.workflow_path()).with(&ctx)?;

        let setting = if paths.setting_path().exists() {
            Some(Setting::from_conf(paths.setting_path()).owe_logic()?)
        } else {
            None
        };
        ctx.with_path("artifact", paths.artifact_path());
        let artifact = ArtifactPackage::from_conf(paths.artifact_path())
            .with(&ctx)
            .owe_logic()?;

        //ctx.with_path("conf_spec", paths.conf_path());
        //let conf_spec = ConfSpec::from_conf(paths.conf_path()).with(&ctx)?;

        ctx.with_path("depends", paths.depends_path());
        let depends = DependencySet::from_conf(paths.depends_path())
            .with(&ctx)
            .owe_logic()?;
        ctx.with_path("vars", paths.vars_path());
        //let vars = VarCollection::eval_from_file(&ValueDict::default(), paths.vars_path())
        let vars = VarCollection::from_conf(paths.vars_path())
            .with(&ctx)
            .owe_logic()?;

        let gxl_prj = GxlProject::load_from(paths.target_root()).with(&ctx)?;
        flag.mark_suc();
        Ok(Self {
            model: target,
            artifact,
            workflow: actions,
            //conf_spec,
            local: Some(target_root.to_path_buf()),
            vars,
            setting,
            depends,
            gxl_prj,
        })
    }
}
impl ModModelSpec {
    pub fn init(
        target: ModelSTD,
        artifact: ArtifactPackage,
        workflow: ModWorkflows,
        gxl_prj: GxlProject,
        //conf_spec: ConfSpec,
        vars: VarCollection,
        setting: Option<Setting>,
    ) -> Self {
        Self {
            model: target,
            workflow,
            gxl_prj,
            artifact,
            local: None,
            vars,
            setting,
            depends: DependencySet::default(),
        }
    }
    pub fn get_local_values(&self, parent: ValuePath) -> MainResult<Option<String>> {
        let value_paths = TargetValuePaths::from(parent.path());
        if value_paths.used_readable().exists() {
            let data = read_to_string(value_paths.used_readable()).owe_sys()?;
            return Ok(Some(data));
        }
        Ok(None)
    }
    pub fn used_value_path(&self) -> MainResult<PathBuf> {
        let local = self
            .local
            .clone()
            .ok_or(MainReason::from(ElementReason::Miss("local-path".into())).to_err())?;
        let value_path = ensure_path(local.join(VALUE_DIR)).owe_logic()?;
        let value_file = value_path.join(USED_JSON);
        Ok(value_file)
    }
}

#[async_trait]
impl Localizable for ModModelSpec {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(target : "/mod/target", "mod-target localize {} success!", self.model()),
            error!(target: "/mod/target", "mod-target localize {} fail!",
                self.local.clone().unwrap_or(PathBuf::from("unknow")).display())
        );
        let mut ctx = WithContext::want("modul localize");
        let local = self.local.clone().ok_or(
            MainReason::from(ElementReason::Miss("local-path".into()))
                .to_err()
                .with(&ctx),
        )?;
        let tpl = local.join(crate::const_vars::SPEC_DIR);
        let localize_path = dst_path.unwrap_or(ValuePath::new(local.join(VALUE_DIR)));

        let value_root = localize_path.path(); //.join(VALUE_DIR);
        let value_paths = TargetValuePaths::from(value_root);
        let used_value_file = self.used_value_path()?;
        let local_path = local.join(LOCAL_DIR);
        debug!( target:"spec/mod/target", "localize mod-target begin: {}" ,local_path.display() );
        make_clean_path(&local_path).owe_logic()?;
        ctx.with_path("dst", &local_path);
        self.crate_sample_value_file(&value_paths)?;
        debug!(target : "/mod/target/loc", "value export");
        let used = self.build_used_value(options, &value_paths)?;
        used.export_origin()
            .save_valconf(value_paths.used_readable())
            .owe_res()?;
        used.export_value().save_json(&used_value_file).owe_res()?;

        debug!(target : "/mod/target/loc", "use value: {}", used_value_file.display());
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
        flag.mark_suc();
        Ok(())
    }
}
