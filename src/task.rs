use derive_getters::Getters;
use derive_more::Display;
use serde::Serialize;

use crate::{error::MainResult, module::ModelSTD};

#[derive(Clone, Debug, PartialEq, Display, Serialize)]
pub enum OperationType {
    #[display("setup")]
    Setup,
    #[display("update")]
    Update,
    #[display("port")]
    Port,
    #[display("backup")]
    Backup,
    #[display("clean")]
    Clean,
    #[display("uninstall")]
    UnInstall,
    Other,
}
pub trait Task {
    fn exec(&self) -> MainResult<()>;
}

pub type TaskHandle = Box<dyn Task>;

pub trait NodeSetupTaskBuilder {
    fn make_setup_task(&self, node: &ModelSTD) -> MainResult<TaskHandle>;
}

pub trait UpdateTaskMaker {
    fn make_update_task(&self) -> MainResult<TaskHandle>;
}

#[derive(Getters)]
pub struct CombinedTask {
    name: String,
    subs: Vec<TaskHandle>,
}
impl CombinedTask {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            subs: Vec::new(),
        }
    }
    pub fn add_sub(&mut self, sub: TaskHandle) {
        self.subs.push(sub);
    }
}
impl Task for CombinedTask {
    fn exec(&self) -> MainResult<()> {
        for task in &self.subs {
            task.exec()?;
        }
        Ok(())
    }
}

pub struct EchoTask {
    cmd: String,
}
impl EchoTask {
    pub fn new<S: Into<String>>(cmd: S) -> Self {
        Self { cmd: cmd.into() }
    }
}

impl Task for EchoTask {
    fn exec(&self) -> MainResult<()> {
        println!("echo task:\n{}\n", self.cmd);
        Ok(())
    }
}
