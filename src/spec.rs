use derive_getters::Getters;
use serde_derive::Deserialize;
use serde_derive::Serialize;
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum FetchTypes {
    Gxl(AgentCode),
    Python(AgentCode),
    Bash(AgentCode),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Getters)]
pub struct AgentCode {
    repo: String,
    file: String,
    func: String,
}
#[allow(dead_code)]
impl AgentCode {
    pub fn new<S: Into<String>>(repo: S, file: S, func: S) -> Self {
        Self {
            repo: repo.into(),
            func: func.into(),
            file: file.into(),
        }
    }

    pub(crate) fn exe_file(&self) -> &str {
        todo!()
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ObsTaskTypes {
    Otl(OtlTask),
    Config(ConfTask),
    WorkLoad(WorkLoadTask),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Getters)]
pub struct OtlTask {
    target: WorkSpec,
}
#[allow(dead_code)]
impl OtlTask {
    pub(crate) fn new(spec: WorkSpec) -> Self {
        Self { target: spec }
    }
}
#[allow(dead_code)]
#[derive(Clone, Debug, Getters)]
pub struct ConfTask {
    target: ConfigTarget,
    fetch: FetchTypes,
}
#[allow(dead_code)]
impl ConfTask {
    pub(crate) fn new(target: ConfigTarget, fetch: FetchTypes) -> Self {
        Self { target, fetch }
    }
}

#[derive(Clone, Debug, Getters)]
pub struct WorkLoadTask {}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum TargetTypes {
    Otl(WorkSpec),
    Config(ConfigTarget),
    WorkLoad,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ConfigTarget {
    relative: String,
}
#[allow(dead_code)]
impl ConfigTarget {
    pub(crate) fn new<S: Into<String>>(relative: S) -> Self {
        Self {
            relative: relative.into(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WorkLoad {
    items: Vec<Metric>,
}

#[derive(Clone, Debug)]
pub struct Metric {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkSpec {
    key: String,
}
impl WorkSpec {
    pub fn new<S: Into<String>>(key: S) -> Self {
        Self { key: key.into() }
    }
}
