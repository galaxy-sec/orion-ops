use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    const_vars::{CONFS_DIR, MOD_DIR},
    types::{Localizable, LocalizePath, UpdateOptions},
    vars::{VarCollection, VarType},
    workflow::prj::GxlProject,
};
use async_trait::async_trait;
use derive_getters::Getters;
use log::{error, info};
use orion_error::{ErrorOwe, ErrorWith};

use crate::{
    addr::{HttpAddr, path_file_name},
    artifact::{Artifact, ArtifactPackage},
    conf::{ConfFile, ConfSpec},
    error::SpecResult,
    resource::CaculateResSpec,
    tools::get_sub_dirs,
    types::{AsyncUpdateable, Persistable},
    workflow::act::ModWorkflows,
};

use super::{
    CpuArch, OsCPE, RunSPC, TargetNode,
    depend::DependencySet,
    init::{ModIniter, ModPrjIniter, mod_init_gitignore},
    setting::Setting,
    target::ModTargetSpec,
};

#[derive(Getters, Clone, Debug)]
pub struct ModuleSpec {
    name: String,
    targets: HashMap<TargetNode, ModTargetSpec>,
    local: Option<PathBuf>,
}
impl ModuleSpec {
    pub fn init<S: Into<String>>(name: S, target_vec: Vec<ModTargetSpec>) -> ModuleSpec {
        let mut targets = HashMap::new();
        for node in target_vec {
            targets.insert(node.target().clone(), node);
        }
        Self {
            name: name.into(),
            targets,
            local: None,
        }
    }
    pub fn clean_other(&mut self, node: &TargetNode) -> SpecResult<()> {
        if let Some(local) = &self.local {
            let src_path = local.join(MOD_DIR);
            let subs = get_sub_dirs(&src_path)?;
            for sub in subs {
                if !sub.ends_with(node.to_string().as_str()) {
                    Self::clean_path(&sub)?;
                }
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
    pub fn save_main(&self, path: &Path, name: Option<String>) -> SpecResult<()> {
        let mod_path = path.join(name.unwrap_or(self.name().clone()));
        std::fs::create_dir_all(&mod_path)
            .owe_conf()
            .with(format!("path: {}", mod_path.display()))?;

        for node in self.targets.values() {
            node.save_main(&mod_path, Some("".into()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl AsyncUpdateable for ModuleSpec {
    async fn update_local(&self, path: &Path, options: &UpdateOptions) -> SpecResult<PathBuf> {
        for (target, node) in &self.targets {
            node.update_local(&path.join(target.to_string()), options)
                .await?;
        }
        Ok(path.to_path_buf())
    }
}

impl Persistable<ModuleSpec> for ModuleSpec {
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()> {
        let mod_path = path.join(name.unwrap_or(self.name().clone()));
        let src_path = mod_path.join(MOD_DIR);
        std::fs::create_dir_all(&mod_path)
            .owe_conf()
            .with(format!("path: {}", mod_path.display()))?;

        mod_init_gitignore(&mod_path)?;
        for node in self.targets.values() {
            node.save_to(&src_path, None)?;
        }

        Ok(())
    }

    fn load_from(path: &Path) -> SpecResult<Self> {
        let name = path_file_name(path)?;
        let name_copy = name.clone();
        let mut flag = log_guard!(
            info!(target: "mod/spec", "load mod-spec {} success!", name_copy ),
            error!(target: "mod/spec", "load mod-spec {} fail!", name_copy)
        );
        let src_path = path.join(MOD_DIR);
        let subs = get_sub_dirs(&src_path)?;
        let mut targets = HashMap::new();
        for sub in subs {
            let node = ModTargetSpec::load_from(&sub).with(&sub)?;
            targets.insert(node.target().clone(), node);
        }
        flag.flag_suc();
        Ok(Self {
            name,
            targets,
            local: Some(path.to_path_buf()),
        })
    }
}

#[async_trait]
impl Localizable for ModuleSpec {
    async fn localize(
        &self,
        dst_path: Option<LocalizePath>,
        global_value: Option<PathBuf>,
    ) -> SpecResult<()> {
        for target in self.targets.values() {
            let target_dst_path = dst_path
                .as_ref()
                .map(|x| x.join_all(PathBuf::from(target.target().to_string())));
            target
                .localize(target_dst_path, global_value.clone())
                .await?;
        }
        Ok(())
    }
}

impl ModuleSpec {
    pub fn for_example() -> Self {
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
        .with_depends(DependencySet::example());

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
        .with_depends(DependencySet::example());
        ModuleSpec::init("postgresql", vec![k8s, host])
    }

    pub fn make_new(name: &str) -> SpecResult<ModuleSpec> {
        let mut conf = ConfSpec::new("1.0.0", CONFS_DIR);
        conf.add(ConfFile::new("example.conf").with_addr(HttpAddr::from(
            "https://mirrors.aliyun.com/postgresql/README",
        )));

        let x86_ubu22_k8s = ModTargetSpec::init(
            TargetNode::x86_ubt22_k8s(),
            ArtifactPackage::from(vec![Artifact::new(
                name,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
                ),
                "postgresql-17.4.tar.gz.md5",
            )]),
            ModWorkflows::mod_k8s_tpl_init(),
            GxlProject::spec_k8s_tpl(),
            //conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
            None,
        );

        let arm_mac_host = ModTargetSpec::init(
            TargetNode::arm_mac14_host(),
            ArtifactPackage::from(vec![Artifact::new(
                name,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
                ),
                "postgresql-17.4.tar.gz.md5",
            )]),
            ModWorkflows::mod_host_tpl_init(),
            GxlProject::spec_host_tpl(),
            //conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
            None,
        );
        let x86_ubt22_host = ModTargetSpec::init(
            TargetNode::arm_mac14_host(),
            ArtifactPackage::from(vec![Artifact::new(
                name,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
                ),
                "postgresql-17.4.tar.gz.md5",
            )]),
            ModWorkflows::mod_host_tpl_init(),
            GxlProject::spec_host_tpl(),
            //conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
            None,
        );

        Ok(ModuleSpec::init(
            name,
            vec![x86_ubu22_k8s, x86_ubt22_host, arm_mac_host],
        ))
    }
}

pub fn make_mod_spec_example() -> SpecResult<ModuleSpec> {
    Ok(ModuleSpec::for_example())
}
pub fn make_mod_spec_4test() -> SpecResult<ModuleSpec> {
    let name = "postgresql";
    let k8s = ModTargetSpec::init(
        TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
        ArtifactPackage::from(vec![Artifact::new(
            name,
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
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

    let host = ModTargetSpec::init(
        TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        ArtifactPackage::from(vec![Artifact::new(
            name,
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
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
    Ok(ModuleSpec::init("postgresql", vec![k8s, host]))
}
