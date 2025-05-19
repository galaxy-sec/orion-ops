use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use async_trait::async_trait;
use derive_getters::Getters;
use log::warn;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, WithContext};
use orion_exchange::vars::{VarCollection, VarType};
use serde_derive::{Deserialize, Serialize};

use crate::{
    addr::{AddrType, LocalAddr, path_file_name},
    artifact::Artifact,
    conf::{ConfFile, ConfSpec},
    error::SpecResult,
    resource::CaculateResSpec,
    software::{FileFormat, LogsSpec},
    task::{EchoTask, NodeSetupTaskBuilder, OperationType, SetupTaskBuilder, TaskHandle},
    types::{AsyncUpdateable, IniAble, SaveAble, TomlAble},
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
        }
    }
}
#[async_trait]
impl AsyncUpdateable for ModuleSpecRef {
    async fn update_local(&self, path: &PathBuf) -> SpecResult<PathBuf> {
        let spec_path = self.addr.update_local(&path).await?;
        let mod_path = path.join(self.name.as_str());
        let spec = ModuleSpec::load_from(&mod_path)?;
        let _x = spec.update_local(&mod_path).await?;
        Ok(spec_path)
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

impl SaveAble<ModuleSpec> for ModuleSpec {
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
impl SaveAble<ModTargetSpec> for ModTargetSpec {
    fn save_to(&self, root: &PathBuf) -> SpecResult<()> {
        let target_path = root.join(self.target());
        std::fs::create_dir_all(&target_path)
            .owe_conf()
            .with(format!("path: {}", target_path.display()))?;
        let artifact_path = target_path.join("artifact.ini");
        self.artifact.save_ini(&artifact_path)?;

        let actions_path = target_path.join("actions");
        self.actions.save_to(&actions_path)?;
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
        let artifact_path = target_path.join("artifact.ini");
        ctx.with("artifact", format!("{}", artifact_path.display()));
        let artifact = Artifact::from_ini(&artifact_path).with(&ctx)?;

        let actions_path = target_path.join("actions");
        let actions = Actions::load_from(&actions_path)?;
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
        conf_spec: ConfSpec,
        res_spec: CaculateResSpec,
        vars: VarCollection,
    ) -> Self {
        Self {
            target: target.into(),
            actions: Actions::tpl_init(),
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
        if let Some(pkg) = self.artifact.af_bin() {
            let shell = format!("gx setup -e local {}", pkg);
            return Ok(Box::new(EchoTask::new(shell)));
        }
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
#[derive(Getters, Clone, Debug)]
pub struct Actions {
    actions: Vec<ActionType>,
}

impl Actions {
    pub fn tpl_init() -> Self {
        let actions = vec![
            ActionType::Gxl(GxlAction::setup_tpl()),
            ActionType::Gxl(GxlAction::update_tpl()),
        ];
        Self { actions }
    }
}

impl SaveAble<Actions> for Actions {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        std::fs::create_dir_all(path)
            .owe_res()
            .with(path.to_string_lossy().to_string())?;
        for item in &self.actions {
            item.save_to(path)?;
        }
        Ok(())
    }

    //加载 path 目录的文件
    fn load_from(path: &PathBuf) -> SpecResult<Actions> {
        let mut actions = Vec::new();
        for entry in std::fs::read_dir(path).owe_res()? {
            let entry = entry.owe_res()?;
            let path = entry.path();

            if path.is_file() {
                let action = ActionType::load_from(&path);
                match action {
                    Ok(act) => {
                        actions.push(act);
                    }
                    Err(e) => {
                        warn!("load ignore : {}\n {}", path.display(), e);
                    }
                }
            }
        }

        Ok(Actions { actions })
    }
}

#[derive(Clone, Debug)]
pub enum ActionType {
    Bash(BashAction),
    Gxl(GxlAction),
}

impl SaveAble<ActionType> for ActionType {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        match self {
            ActionType::Bash(act) => act.save_to(path),
            ActionType::Gxl(act) => act.save_to(path),
        }
    }

    fn load_from(path: &PathBuf) -> SpecResult<ActionType> {
        if path.ends_with(".sh") {
            return Ok(ActionType::Bash(BashAction::load_from(path)?));
        }
        if path.ends_with(".gxl") {
            return Ok(ActionType::Gxl(GxlAction::load_from(path)?));
        }
        Err(StructError::from_conf(format!(
            "bad filename :{}",
            path.display()
        )))
    }
}

const SETUP_GXL: &str = include_str!("init/actions/setup.gxl");

#[derive(Getters, Clone, Debug)]
pub struct GxlAction {
    task: OperationType,
    code: String,
}

impl GxlAction {
    pub fn setup_tpl() -> Self {
        Self {
            task: OperationType::Setup,
            code: SETUP_GXL.to_string(),
        }
    }
    pub fn update_tpl() -> Self {
        Self {
            task: OperationType::Update,
            code: SETUP_GXL.to_string(),
        }
    }
}
impl SaveAble<GxlAction> for GxlAction {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let path_file = path.join("setup.gxl");
        std::fs::write(path_file, self.code.as_str()).owe_res()?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<GxlAction> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad file name".to_string()))?;

        let task_type = match file_name {
            "setup.gxl" => OperationType::Setup,
            "update.gxl" => OperationType::Update,
            _ => todo!(),
        };
        let code = std::fs::read_to_string(path).owe_res()?;
        Ok(Self {
            task: task_type,
            code,
        })
    }
}

const SETUP_SH: &str = include_str!("init/actions/setup.sh");

#[derive(Getters, Clone, Debug)]
pub struct BashAction {
    task: OperationType,
    code: String,
}

impl BashAction {
    pub fn setup_tpl() -> Self {
        Self {
            task: OperationType::Setup,
            code: SETUP_SH.to_string(),
        }
    }
}

impl SaveAble<BashAction> for BashAction {
    fn save_to(&self, path: &PathBuf) -> SpecResult<()> {
        let path_file = path.join("setup.sh");
        std::fs::write(path_file, self.code.as_str()).owe_res()?;
        Ok(())
    }

    fn load_from(path: &PathBuf) -> SpecResult<Self> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| StructError::from_conf("bad file name".to_string()))?;

        let task_type = match file_name {
            "setup.sh" => OperationType::Setup,
            "update.sh" => OperationType::Update,
            _ => todo!(),
        };
        let code = std::fs::read_to_string(path).owe_res()?;
        Ok(Self {
            task: task_type,
            code,
        })
    }
}

pub fn make_mod_spec_example() -> SpecResult<ModuleSpec> {
    let mut conf = ConfSpec::new("1.0.0");
    conf.add(
        ConfFile::new(FileFormat::Dsl, "my.cnf")
            .with_addr(LocalAddr::from("./example/knowlege/mysql/my.cnf")),
    );

    let k8s = ModTargetSpec::init(
        "k8s",
        Artifact::from(("mysql-4.0", "mysql::latest", "${HOME}/Devspace/mysql")),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
    );

    let host = ModTargetSpec::init(
        "host",
        Artifact::from(("mysql-4.0", "mysql::latest", "${HOME}/Devspace/mysql")),
        conf.clone(),
        CaculateResSpec::new(2, 4),
        VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
    );
    Ok(ModuleSpec::init("mysql", k8s, host))
}

#[cfg(test)]
pub mod test {
    use std::net::Ipv4Addr;

    use orion_exchange::vars::VarType;

    use crate::{
        addr::LocalAddr, conf::ConfFile, const_vars::MODULES_SPC_ROOT, error::SpecResult,
        software::FileFormat, system::NetResSpace,
    };

    use super::*;

    pub fn make_mod_spec_warp() -> SpecResult<ModuleSpec> {
        let _net = NetResSpace::new(
            Ipv4Addr::new(10, 0, 0, 1),
            (Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 10)),
        );

        let warp_conf = ConfSpec::from_files(vec![
            (FileFormat::Toml, "./conf/dvron.toml"),
            (FileFormat::Dsl, "./conf/dvadm.oml"),
            (FileFormat::Toml, "./conf/knowdb.toml"),
            (FileFormat::Toml, "./sink/framework.toml"),
            (FileFormat::Toml, "./sink/privacy.toml"),
            (FileFormat::Toml, "./source/dysrc.toml"),
        ]);
        let k8s = ModTargetSpec::init(
            "k8s",
            Artifact::from((
                "warp-7",
                "warp-flow::latest",
                "${HOME}/Devspace/dy-sec/warp-flow/target/release/wpflow",
            )),
            warp_conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );

        let host = ModTargetSpec::init(
            "host",
            Artifact::from((
                "warp-7",
                "warp-flow::latest",
                "${HOME}/Devspace/dy-sec/warp-flow/target/release/wpflow",
            )),
            warp_conf,
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );

        Ok(ModuleSpec::init("warpflow", k8s, host))
    }
    pub fn make_mod_spec_mysql() -> SpecResult<ModuleSpec> {
        let mut conf = ConfSpec::new("1.0.0");
        conf.add(
            ConfFile::new(FileFormat::Dsl, "my.cnf")
                .with_addr(LocalAddr::from("./example/knowlege/mysql/my.cnf")),
        );

        let k8s = ModTargetSpec::init(
            "k8s",
            Artifact::from(("mysql-4.0", "mysql::latest", "${HOME}/Devspace/mysql")),
            conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );

        let host = ModTargetSpec::init(
            "host",
            Artifact::from(("mysql-4.0", "mysql::latest", "${HOME}/Devspace/mysql")),
            conf.clone(),
            CaculateResSpec::new(2, 4),
            VarCollection::define(vec![VarType::from(("SPEED_LIMIT", 1000))]),
        );
        Ok(ModuleSpec::init("mysql", k8s, host))
    }

    #[test]
    fn build_mod_warpflow() -> SpecResult<()> {
        let spec = make_mod_spec_warp()?;
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT))?;
        let _loaded = ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join("warpflow"))?;
        let spec = make_mod_spec_mysql()?;
        spec.save_to(&PathBuf::from(MODULES_SPC_ROOT))?;
        let _loaded = ModuleSpec::load_from(&PathBuf::from(MODULES_SPC_ROOT).join("mysql"))?;
        Ok(())
    }
}
