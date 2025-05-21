use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use async_trait::async_trait;
use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith, WithContext};
use orion_exchange::vars::{VarCollection, VarType};
use serde_derive::{Deserialize, Serialize};

use crate::{
    action::act::Actions,
    addr::{AddrType, HttpAddr, path_file_name},
    artifact::{Artifact, OsType},
    conf::{ConfFile, ConfSpec},
    error::SpecResult,
    resource::CaculateResSpec,
    software::LogsSpec,
    task::{NodeSetupTaskBuilder, SetupTaskBuilder, TaskHandle},
    types::{AsyncUpdateable, Persistable, TomlAble},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    Host,
    K8s,
}
impl FromStr for NodeType {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "host" => Ok(NodeType::Host),
            "k8s" => Ok(NodeType::K8s),
            _ => Err(()),
        }
    }
}
impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Host => write!(f, "host"),
            NodeType::K8s => write!(f, "k8s"),
        }
    }
}
#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct ModuleSpecRef {
    name: String,
    addr: AddrType,
    node: NodeType,
    effective: Option<bool>,
}

impl ModuleSpecRef {
    pub fn from<S: Into<String>, A: Into<AddrType>>(
        name: S,
        addr: A,
        node: NodeType,
    ) -> ModuleSpecRef {
        Self {
            name: name.into(),
            addr: addr.into(),
            node,
            effective: None,
        }
    }
    pub fn with_effective(mut self, effective: bool) -> Self {
        self.effective = Some(effective);
        self
    }
}
#[async_trait]
impl AsyncUpdateable for ModuleSpecRef {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        if self.effective.is_none_or(|x| x) {
            let spec_path = self.addr.update_local(&path).await?;
            let mod_path = path.join(self.name.as_str());
            let spec = ModuleSpec::load_from(&mod_path)?;
            let _x = spec.update_local(&mod_path).await?;
            Ok(spec_path)
        } else {
            Ok(path.clone())
        }
    }

    async fn update_rename(&self, path: &PathBuf, name: &str) -> SpecResult<()> {
        self.addr.update_rename(&path, name).await
    }
}

#[derive(Getters, Clone, Debug)]
pub struct ModTargetSpec {
    target: String,
    artifact: Artifact,
    actions: Actions,
    conf_spec: ConfSpec,
    logs_spec: LogsSpec,
    res_spec: CaculateResSpec,
    vars: VarCollection,
}

#[derive(Getters, Clone, Debug)]
pub struct ModuleSpec {
    name: String,
    k8s: ModTargetSpec,
    host: ModTargetSpec,
}
impl ModuleSpec {
    pub fn init<S: Into<String>>(name: S, k8s: ModTargetSpec, host: ModTargetSpec) -> ModuleSpec {
        Self {
            name: name.into(),
            k8s,
            host,
        }
    }
}

#[async_trait]
impl AsyncUpdateable for ModuleSpec {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        self.host.update_local(&path.join("host")).await?;
        self.k8s.update_local(&path.join("k8s")).await?;
        Ok(path.clone())
    }
}

#[async_trait]
impl AsyncUpdateable for ModTargetSpec {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        self.conf_spec.update_local(path).await?;
        Ok(path.clone())
    }
}

impl Persistable<ModuleSpec> for ModuleSpec {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let mod_path = path.join(self.name());
        std::fs::create_dir_all(&mod_path)
            .owe_conf()
            .with(format!("path: {}", mod_path.display()))?;
        self.k8s.save_to(&mod_path)?;
        self.host.save_to(&mod_path)?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<Self> {
        let name = path_file_name(&path)?;
        let k8s = ModTargetSpec::load_from(&path.join("k8s"))?;
        let host = ModTargetSpec::load_from(&path.join("host"))?;
        Ok(Self { name, k8s, host })
    }
}
impl Persistable<ModTargetSpec> for ModTargetSpec {
    fn save_to(&self, root: &PathBuf) -> SpecResult<()> {
        let target_path = root.join(self.target());
        std::fs::create_dir_all(&target_path)
            .owe_conf()
            .with(format!("path: {}", target_path.display()))?;
        let artifact_path = target_path.join("artifact.toml");
        self.artifact.save_toml(&artifact_path)?;

        self.actions.save_to(&target_path)?;
        let spec_path = target_path.join("conf_spec.toml");
        self.conf_spec.save_toml(&spec_path)?;
        let spec_path = target_path.join("logs_spec.toml");
        self.logs_spec.save_toml(&spec_path)?;

        let spec_path = target_path.join("res_spec.toml");
        self.res_spec.save_toml(&spec_path)?;
        let vars_path = target_path.join("vars.toml");
        self.vars.save_toml(&vars_path)?;
        Ok(())
    }

    fn load_from(target_path: &PathBuf) -> SpecResult<Self> {
        //target: &str
        let mut ctx = WithContext::want("load mod spec");
        //let target_path = root.join(target);
        let artifact_path = target_path.join("artifact.toml");
        ctx.with("artifact", format!("{}", artifact_path.display()));
        let artifact = Artifact::from_toml(&artifact_path).with(&ctx)?;

        let actions = Actions::load_from(&target_path)?;
        let spec_path = target_path.join("conf_spec.toml");
        let conf_spec = ConfSpec::from_toml(&spec_path)?;
        let spec_path = target_path.join("logs_spec.toml");
        let logs_spec = LogsSpec::from_toml(&spec_path)?;
        let spec_path = target_path.join("res_spec.toml");
        let res_spec = CaculateResSpec::from_toml(&spec_path)?;
        let vars_path = target_path.join("vars.toml");
        let vars = VarCollection::from_toml(&vars_path)?;
        let target = path_file_name(target_path)?;

        Ok(Self {
            target,
            artifact,
            actions,
            conf_spec,
            logs_spec,
            res_spec,
            vars,
        })
    }
}
impl ModTargetSpec {
    pub fn init<S: Into<String>>(
        target: S,
        artifact: Artifact,
        actions: Actions,
        conf_spec: ConfSpec,
        res_spec: CaculateResSpec,
        vars: VarCollection,
    ) -> Self {
        Self {
            target: target.into(),
            actions,
            artifact,
            conf_spec,
            logs_spec: LogsSpec::tpl_init(),
            res_spec,
            vars,
        }
    }
}

impl NodeSetupTaskBuilder for ModuleSpec {
    fn make_setup_task(&self, node: &NodeType) -> SpecResult<TaskHandle> {
        match node {
            NodeType::Host => self.host.make_setup_task(),
            NodeType::K8s => self.k8s.make_setup_task(),
        }
    }
}

impl SetupTaskBuilder for ModTargetSpec {
    fn make_setup_task(&self) -> SpecResult<TaskHandle> {
        todo!()
    }
}

#[derive(Getters, Clone, Debug)]
pub struct WorkLoad {
    metrics: HashMap<String, bool>,
}
impl WorkLoad {
    pub fn tpl_init() -> Self {
        let mut metrics = HashMap::new();
        metrics.insert("request".to_string(), true);
        Self { metrics }
    }
}

pub fn make_mod_spec_example() -> SpecResult<ModuleSpec> {
    let mut conf = ConfSpec::new("1.0.0");
    conf.add(ConfFile::new("postgresql.conf").with_addr(HttpAddr::from(
        "https://github.com/galaxy-sec/module-specs/blob/main/postgresql/conf/postgresql.conf",
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
    Ok(ModuleSpec::init("postgresql", k8s, host))
}

#[cfg(test)]
pub mod test {
    use std::net::Ipv4Addr;

    use orion_exchange::vars::VarType;

    use crate::{
        addr::LocalAddr, conf::ConfFile, const_vars::MODULES_SPC_ROOT, error::SpecResult,
        system::NetResSpace,
    };

    use super::*;

    pub fn make_mod_spec_warp() -> SpecResult<ModuleSpec> {
        let _net = NetResSpace::new(
            Ipv4Addr::new(10, 0, 0, 1),
            (Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 10)),
        );

        let warp_conf = ConfSpec::from_files(vec![
            "./conf/dvron.toml",
            "./conf/dvadm.oml",
            "./conf/knowdb.toml",
            "./sink/framework.toml",
            "./sink/privacy.toml",
            "./source/dysrc.toml",
        ]);
        let k8s = ModTargetSpec::init(
            "k8s",
            Artifact::new(
                "warp-7",
                OsType::Ubuntu,
                LocalAddr::from("${HOME}/Devspace/dy-sec/warp-flow/target/release/wpflow"),
            ),
            Actions::k8s_tpl_init(),
            warp_conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );

        let host = ModTargetSpec::init(
            "host",
            Artifact::new(
                "warp-7",
                OsType::MacOs,
                LocalAddr::from("${HOME}/Devspace/dy-sec/warp-flow/target/release/wpflow"),
            ),
            Actions::host_tpl_init(),
            warp_conf,
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );

        Ok(ModuleSpec::init("warpflow", k8s, host))
    }
    pub fn make_mod_spec_mysql() -> SpecResult<ModuleSpec> {
        let mut conf = ConfSpec::new("1.0.0");
        conf.add(
            ConfFile::new("my.cnf").with_addr(LocalAddr::from("./example/knowlege/mysql/my.cnf")),
        );

        let k8s = ModTargetSpec::init(
            "k8s",
            Artifact::new(
                "mysql-4.0",
                OsType::Ubuntu,
                LocalAddr::from("${HOME}/Devspace/mysql"),
            ),
            Actions::k8s_tpl_init(),
            conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );

        let host = ModTargetSpec::init(
            "host",
            Artifact::new(
                "mysql-4.0",
                OsType::MacOs,
                LocalAddr::from("${HOME}/Devspace/mysql"),
            ),
            Actions::host_tpl_init(),
            conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );
        Ok(ModuleSpec::init("mysql", k8s, host))
    }

    #[test]
    fn build_mod_example() -> SpecResult<()> {
        let spec = make_mod_spec_example()?;
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT))?;
        let _loaded = ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join(spec.name()))?;
        let spec = make_mod_spec_mysql()?;
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT))?;
        let _loaded = ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join(spec.name()))?;
        Ok(())
    }
}
