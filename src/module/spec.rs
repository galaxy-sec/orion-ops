use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    const_vars::CONFS_DIR,
    error::ElementReason,
    types::{Localizable, LocalizePath},
    vars::{VarCollection, VarType},
};
use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith};

use crate::{
    action::act::ModWorkflows,
    addr::{HttpAddr, path_file_name},
    artifact::{Artifact, ArtifactPackage, OsType},
    conf::{ConfFile, ConfSpec},
    error::{SpecReason, SpecResult, ToErr},
    resource::CaculateResSpec,
    task::{NodeSetupTaskBuilder, SetupTaskBuilder, TaskHandle},
    tools::get_sub_dirs,
    types::{AsyncUpdateable, Persistable},
};

use super::{
    CpuArch, OsCPE, RunSPC, TargetNode,
    init::{ModIniter, mod_init_gitignore},
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
            let subs = get_sub_dirs(local)?;
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
    async fn update_local(&self, path: &Path) -> SpecResult<PathBuf> {
        for (target, node) in &self.targets {
            node.update_local(&path.join(target.to_string())).await?;
        }
        Ok(path.to_path_buf())
    }
}

impl Persistable<ModuleSpec> for ModuleSpec {
    fn save_to(&self, path: &Path, name: Option<String>) -> SpecResult<()> {
        let mod_path = path.join(name.unwrap_or(self.name().clone()));
        std::fs::create_dir_all(&mod_path)
            .owe_conf()
            .with(format!("path: {}", mod_path.display()))?;

        mod_init_gitignore(&mod_path)?;
        for node in self.targets.values() {
            node.save_to(&mod_path, None)?;
        }

        Ok(())
    }

    fn load_from(path: &Path) -> SpecResult<Self> {
        let name = path_file_name(path)?;
        let subs = get_sub_dirs(path)?;
        let mut targets = HashMap::new();
        for sub in subs {
            let node = ModTargetSpec::load_from(&sub).with(&sub)?;
            targets.insert(node.target().clone(), node);
        }
        Ok(Self {
            name,
            targets,
            local: Some(path.to_path_buf()),
        })
    }
}
impl NodeSetupTaskBuilder for ModuleSpec {
    fn make_setup_task(&self, node: &TargetNode) -> SpecResult<TaskHandle> {
        if let Some(cur_node) = self.targets().get(node) {
            cur_node.make_setup_task()
        } else {
            SpecReason::from(ElementReason::Miss(node.to_string())).err_result()
        }
    }
}

impl SetupTaskBuilder for ModTargetSpec {
    fn make_setup_task(&self) -> SpecResult<TaskHandle> {
        todo!()
    }
}

#[async_trait]
impl Localizable for ModuleSpec {
    async fn localize(&self, dst_path: Option<LocalizePath>) -> SpecResult<()> {
        for target in self.targets.values() {
            target.localize(dst_path.clone()).await?;
        }
        Ok(())
    }
}

pub fn make_mod_spec_new(name: &str) -> SpecResult<ModuleSpec> {
    let mut conf = ConfSpec::new("1.0.0", CONFS_DIR);
    conf.add(ConfFile::new("example.conf").with_addr(HttpAddr::from(
        "https://mirrors.aliyun.com/postgresql/README",
    )));

    let cpe = name;
    let k8s = ModTargetSpec::init(
        TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
        ArtifactPackage::from(vec![
            Artifact::new(
                cpe,
                OsType::MacOs,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
                ),
                "postgresql-17.4.tar.gz.md5",
            ),
            Artifact::new(
                cpe,
                OsType::MacOs,
                HttpAddr::from(
                    "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.3.tar.gz.md5",
                ),
                "postgresql-17.3.tar.gz.md5",
            ),
        ]),
        ModWorkflows::mod_k8s_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
        None,
    );

    let host = ModTargetSpec::init(
        TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        ArtifactPackage::from(vec![Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from(
                "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
            ),
            "postgresql-17.4.tar.gz.md5",
        )]),
        ModWorkflows::mod_host_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
        None,
    );
    Ok(ModuleSpec::init(cpe, vec![k8s, host]))
}

pub fn make_mod_spec_example() -> SpecResult<ModuleSpec> {
    let mut conf = ConfSpec::new("1.0.0", CONFS_DIR);
    conf.add(ConfFile::new("postgresql.conf").with_addr(HttpAddr::from(
        "https://mirrors.aliyun.com/postgresql/README",
    )));

    let cpe = "postgresql";
    let k8s = ModTargetSpec::init(
        TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
        ArtifactPackage::from(vec![Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
            "postgresql-17.4.tar.gz",
        )]),
        ModWorkflows::mod_k8s_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        Some(Setting::example()),
    );

    let host = ModTargetSpec::init(
        TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        ArtifactPackage::from(vec![Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
            "postgresql-17.4.tar.gz",
        )]),
        ModWorkflows::mod_host_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        Some(Setting::example()),
    );
    Ok(ModuleSpec::init("postgresql", vec![k8s, host]))
}

#[cfg(test)]
pub mod test {

    use orion_error::TestAssert;

    use crate::{const_vars::MODULES_SPC_ROOT, error::SpecResult, tools::test_init};

    use super::*;

    pub fn make_mod_spec_mod1() -> SpecResult<ModuleSpec> {
        make_mod_spec_new("example_mod1")
    }

    #[tokio::test]
    async fn build_mod_example() -> SpecResult<()> {
        test_init();
        let spec = make_mod_spec_example().assert();
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT), None)
            .assert();
        let loaded =
            ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join(spec.name())).assert();
        loaded.localize(None).await.assert();
        let spec = make_mod_spec_mod1().assert();
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT), None)?;
        let loaded =
            ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join(spec.name())).assert();
        loaded.localize(None).await.assert();
        Ok(())
    }
}
