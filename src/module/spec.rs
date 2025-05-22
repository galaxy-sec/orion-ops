use std::path::PathBuf;

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith};
use orion_exchange::vars::{VarCollection, VarType};

use crate::{
    action::act::Actions,
    addr::{HttpAddr, path_file_name},
    artifact::{Artifact, OsType},
    conf::{ConfFile, ConfSpec},
    error::{SpecReason, SpecResult, ToErr},
    resource::CaculateResSpec,
    task::{NodeSetupTaskBuilder, SetupTaskBuilder, TaskHandle},
    types::{AsyncUpdateable, Persistable},
};

use super::{NodeType, target::ModTargetSpec};

#[derive(Getters, Clone, Debug)]
pub struct ModuleSpec {
    name: String,
    k8s: Option<ModTargetSpec>,
    host: Option<ModTargetSpec>,
    local: Option<PathBuf>,
}
impl ModuleSpec {
    pub fn init<S: Into<String>>(
        name: S,
        k8s: Option<ModTargetSpec>,
        host: Option<ModTargetSpec>,
    ) -> ModuleSpec {
        Self {
            name: name.into(),
            k8s,
            host,
            local: None,
        }
    }
    pub fn clean_other(&mut self, node: &NodeType) -> SpecResult<()> {
        match node {
            NodeType::Host => {
                self.host = None;
                self.local
                    .as_ref()
                    .map(|x| x.join("k8s"))
                    .map(Self::clean_path);
            }
            NodeType::K8s => {
                self.k8s = None;
                self.local
                    .as_ref()
                    .map(|x| x.join("host"))
                    .map(Self::clean_path);
            }
        }
        Ok(())
    }
    fn clean_path(path: PathBuf) -> SpecResult<()> {
        if path.exists() {
            std::fs::remove_dir_all(&path).owe_res().with(&path)?;
        }
        Ok(())
    }
}

#[async_trait]
impl AsyncUpdateable for ModuleSpec {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        if let Some(host) = &self.host {
            host.update_local(&path.join("host")).await?;
        }
        if let Some(k8s) = &self.k8s {
            k8s.update_local(&path.join("k8s")).await?;
        }
        Ok(path.clone())
    }
}

impl Persistable<ModuleSpec> for ModuleSpec {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let mod_path = path.join(self.name());
        std::fs::create_dir_all(&mod_path)
            .owe_conf()
            .with(format!("path: {}", mod_path.display()))?;
        if let Some(host) = &self.host {
            host.save_to(&mod_path)?;
        }
        if let Some(k8s) = &self.k8s {
            k8s.save_to(&mod_path)?;
        }
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<Self> {
        let name = path_file_name(&path)?;
        let k8s_path = path.join("k8s");
        let host_path = path.join("host");
        let k8s = if k8s_path.exists() {
            Some(ModTargetSpec::load_from(&path.join("k8s"))?)
        } else {
            None
        };
        let host = if host_path.exists() {
            Some(ModTargetSpec::load_from(&path.join("host"))?)
        } else {
            None
        };
        Ok(Self {
            name,
            k8s,
            host,
            local: Some(path.clone()),
        })
    }
}
impl NodeSetupTaskBuilder for ModuleSpec {
    fn make_setup_task(&self, node: &NodeType) -> SpecResult<TaskHandle> {
        match node {
            NodeType::Host => self
                .host
                .as_ref()
                .map(|x| x.make_setup_task())
                .ok_or(SpecReason::Miss("host spec".into()).to_err())?,
            NodeType::K8s => self
                .k8s
                .as_ref()
                .map(|x| x.make_setup_task())
                .ok_or(SpecReason::Miss("k8s spec".into()).to_err())?,
        }
    }
}

impl SetupTaskBuilder for ModTargetSpec {
    fn make_setup_task(&self) -> SpecResult<TaskHandle> {
        todo!()
    }
}

pub fn make_mod_spec_new(name: &str) -> SpecResult<ModuleSpec> {
    let mut conf = ConfSpec::new("1.0.0");
    conf.add(ConfFile::new("example.conf").with_addr(HttpAddr::from(
        "https://mirrors.aliyun.com/postgresql/README",
    )));

    let cpe = name;
    let k8s = ModTargetSpec::init(
        "k8s",
        Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from(
                "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
            ),
        ),
        Actions::k8s_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
    );

    let host = ModTargetSpec::init(
        "host",
        Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from(
                "https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz.md5",
            ),
        ),
        Actions::host_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("EXAMPLE_SIZE", 1000))]),
    );
    Ok(ModuleSpec::init(cpe, Some(k8s), Some(host)))
}

pub fn make_mod_spec_example() -> SpecResult<ModuleSpec> {
    let mut conf = ConfSpec::new("1.0.0");
    conf.add(ConfFile::new("postgresql.conf").with_addr(HttpAddr::from(
        "https://raw.githubusercontent.com/galaxy-sec/module-specs/refs/heads/main/postgresql/conf/postgresql.conf"
    )));

    let cpe = "postgresql";
    let k8s = ModTargetSpec::init(
        "k8s",
        Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
        ),
        Actions::k8s_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
    );

    let host = ModTargetSpec::init(
        "host",
        Artifact::new(
            cpe,
            OsType::MacOs,
            HttpAddr::from("https://mirrors.aliyun.com/postgresql/latest/postgresql-17.4.tar.gz"),
        ),
        Actions::host_tpl_init(),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
    );
    Ok(ModuleSpec::init("postgresql", Some(k8s), Some(host)))
}

#[cfg(test)]
pub mod test {

    use crate::{const_vars::MODULES_SPC_ROOT, error::SpecResult};

    use super::*;

    pub fn make_mod_spec_mod1() -> SpecResult<ModuleSpec> {
        make_mod_spec_new("example_mod1")
    }

    #[test]
    fn build_mod_example() -> SpecResult<()> {
        let spec = make_mod_spec_example()?;
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT))?;
        let _loaded = ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join(spec.name()))?;
        let spec = make_mod_spec_mod1()?;
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT))?;
        let _loaded = ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join(spec.name()))?;
        Ok(())
    }
}
