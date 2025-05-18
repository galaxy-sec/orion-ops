use std::fmt::Debug;
use std::fmt::Display;
use std::fs::read_to_string;

use derive_getters::Getters;
use log::warn;
use tracing::debug;
use tracing::info;

use crate::resource::ResAddress;
use crate::resource::ResouceTypes;
use crate::resource::ResourceNode;
use crate::software::SoftWare;
use crate::spec::WorkSpec;

#[derive(Getters, Debug)]
pub struct WorkNode {
    spec: WorkSpec,
    res: ResourceNode,
    wins: Vec<WorkInstance>,
}

#[derive(Getters, Debug)]
pub struct WorkInstance {
    root: String,
    spec: WorkSpec,
    soft: SoftWare,
    res: ResouceTypes,
}
impl WorkInstance {
    fn new<S: Into<String>>(root: S, spec: WorkSpec, soft: SoftWare, res: ResouceTypes) -> Self {
        Self {
            root: root.into(),
            spec,
            soft,
            res,
        }
    }
}

impl WorkNode {
    pub fn new(spec: WorkSpec, res: ResourceNode) -> Self {
        Self {
            spec,
            res,
            wins: Vec::new(),
        }
    }

    pub fn setup(&mut self, soft: SoftWare) {
        //self.softs.push(soft);
        for res in self.res.items() {
            let wins =
                WorkInstance::new("/usr/local", self.spec.clone(), soft.clone(), res.clone());
            self.wins.push(wins)
        }
    }
}

impl WorkSpec {
    pub fn from_file(path: &str) -> Self {
        let file_data = read_to_string(path).unwrap();
        let spec: WorkSpec = toml::from_str(file_data.as_str()).unwrap();
        spec
    }
}

#[derive(Default, Clone, Debug)]
pub struct ObserveCommander {}
impl ObserveCommander {
    pub fn build_target(&self, node: WorkNode) -> Vec<ObsPackage> {
        let ins_vec = node.wins();
        let mut obs_vec = Vec::new();
        for ins in ins_vec {
            let confspec = ins.soft().confspec();

            let mut package = ObsPackage::new(ins.res().address().clone());
            for item in confspec.files() {
                //item.relative()
                let obs_task = ObsTaskTypes::Config(ConfTask::new(
                    ConfigTarget::new(item.path()),
                    FetchTypes::Bash(AgentCode::new("os", "os.sh", "copy_upload")),
                ));
                package.add(obs_task);

                let obs_target = ObsTaskTypes::Otl(OtlTask::new(ins.soft().workspec().clone()));
                package.add(obs_target);
            }
            info!("build task package to {}", ins.res().address());
            obs_vec.push(package)
        }
        obs_vec
    }
}

#[derive(Clone, Debug)]
pub struct ObserveService {
    sender: task_queue::Sender,
    pkgs: Vec<ObsPackage>,
}
impl ObserveService {
    pub fn add_target(&mut self, mut pkgs: Vec<ObsPackage>) {
        self.pkgs.append(&mut pkgs);
    }

    pub async fn work(&self) {
        //TODO：立即发送
        //TODO: 定期执行
        for pkg in &self.pkgs {
            info!("dispatch task package");
            self.sender.send(pkg.clone()).await;
        }
    }

    pub fn new(sender: task_queue::Sender) -> Self {
        Self {
            sender,
            pkgs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Getters)]
pub(crate) struct ObsPackage {
    address: ResAddress,
    targes: Vec<ObsTaskTypes>,
}
impl ObsPackage {
    fn add(&mut self, obs_target: ObsTaskTypes) {
        self.targes.push(obs_target);
    }

    fn new(address: ResAddress) -> Self {
        Self {
            address,
            targes: Vec::new(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct LocalWorker {}
impl LocalWorker {
    pub async fn execute(&self, task: &ConfTask, data_sender: &raw_queue::Sender) {
        match task.fetch() {
            FetchTypes::Bash(code) => {
                GitRepo::pull(code.repo());
                let json = GxShell::exec(code.file(), code.func());
                info!("task result: {}", json);
                data_sender.send(json).await;
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct CollectService {
    data_sender: raw_queue::Sender,
}
impl CollectService {
    pub fn new(data_sender: raw_queue::Sender) -> Self {
        Self { data_sender }
    }

    pub async fn work(&self, mut cmd_receiver: task_queue::Receiver) {
        let worker = LocalWorker::default();
        loop {
            if let Some(pkg) = cmd_receiver.recv().await {
                debug!("collect receiver task package ");
                for task in pkg.targes() {
                    match task {
                        ObsTaskTypes::Config(task) => {
                            info!("proc config task ");
                            worker.execute(&task, &self.data_sender).await;
                        }
                        ObsTaskTypes::Otl(_otl_task) => {
                            warn!("OTL not yet implemented ");
                        }
                        ObsTaskTypes::WorkLoad(_work_load_task) => todo!(),
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ReceiveService {
    dat_sender: data_queue::Sender,
}
impl ReceiveService {
    pub async fn work(&self, mut receiver: raw_queue::Receiver) {
        loop {
            if let Some(data) = receiver.recv().await {
                self.dat_sender.send(data).await;
            }
        }
    }

    pub fn new(sender: data_queue::Sender) -> Self {
        Self { dat_sender: sender }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ObsDataStore {}
impl ObsDataStore {}
#[derive(Debug)]
pub struct TrapService {
    warn_sender: warn_queue::Sender,
}
impl TrapService {
    pub async fn work(&self, mut data_receiver: data_queue::Receiver) {
        loop {
            if let Some(data) = data_receiver.recv().await {
                let record = TrapRecord::new(AlertLevel::Warn, data);
                self.warn_sender.send(record).await;
            }
        }
    }

    pub fn new(warn_sender: warn_queue::Sender) -> Self {
        Self { warn_sender }
    }
}

#[derive(Clone, Debug)]
pub enum AlertLevel {
    Warn,
    Notice,
}
#[derive(Clone, Debug)]
pub struct TrapRecord {
    level: AlertLevel,
    data: String,
}
impl TrapRecord {
    fn new(level: AlertLevel, data: String) -> Self {
        Self { level, data }
    }
}
impl Display for TrapRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.level)?;
        writeln!(f, "{}", self.data)
    }
}

#[derive(Clone, Debug)]
pub struct ReportService {}

impl ReportService {
    pub async fn work(&self, mut warn_receiver: warn_queue::Receiver) {
        loop {
            if let Some(rec) = warn_receiver.recv().await {
                println!("trap rec\n {}", rec);
            }
        }
    }

    pub(crate) fn new() -> Self {
        Self {}
    }
}
