use crate::{artifact::Artifact, conf::ConfSpecRef, spec::WorkSpec};
use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct SoftWare {
    workspec: WorkSpec,
    artifact: Artifact,
    confspec: ConfSpecRef,
}
impl SoftWare {
    pub fn new(artifact: Artifact, workspec: WorkSpec, confspec: ConfSpecRef) -> Self {
        Self {
            workspec,
            artifact,
            confspec,
        }
    }
}

#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct LogFile {
    path: String,
}

impl LogFile {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }
}

//rules: Vec<ConstraintRule>,

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum FileFormat {
    Json,
    Toml,
    Yaml,
    Dsl,
}

#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct LogsSpec {
    version: String,
    files: Vec<LogFile>,
}
impl LogsSpec {
    pub(crate) fn tpl_init() -> LogsSpec {
        Self {
            version: "0.1.0".to_string(),
            files: vec![LogFile::new("logs/log*")],
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
pub struct ConstraintRule {
    key: String,
    constraint: Constraint,
}

impl ConstraintRule {
    pub fn new<S: Into<String>>(key: S, constraint: Constraint) -> Self {
        Self {
            key: key.into(),
            constraint,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Constraint {
    Matching(String),
    WithInScope(u64, u64),
}

#[allow(dead_code)]
pub fn redis_confspec() -> ConfSpecRef {
    let conf_path = "./spec/redis/config_spec.toml";
    ConfSpecRef::new(conf_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use orion_error::ErrorOwe;

    use crate::{
        addr::LocalAddr,
        artifact::AfType,
        conf::{ConfFile, ConfSpec},
        error::SpecResult,
        types::TomlAble,
    };

    use super::*;

    // 基础功能测试
    #[test]
    fn test_conf_spec_creation() {
        let spec = ConfSpec::new("1.0");
        assert_eq!(spec.version(), "1.0");
    }

    // 序列化测试
    #[test]
    fn confspec_save_load() -> SpecResult<()> {
        let root_path = PathBuf::from("./example/spec/redis");
        std::fs::create_dir_all(&root_path).owe_res()?;
        let mut redis = ConfSpec::new("1.0");
        redis.add(ConfFile::new(FileFormat::Dsl, "./nginx.conf"));

        let path = root_path.join("config_spec.toml");
        redis.save_toml(&path).unwrap();
        let loaded = ConfSpec::from_toml(&path).unwrap();
        assert_eq!(redis.version(), loaded.version());

        let warpflow = ConfSpec::from_files(vec![
            (FileFormat::Toml, "./conf/dvron.toml"),
            (FileFormat::Toml, "./conf/dvgen.toml"),
            (FileFormat::Toml, "./sink/framework.toml"),
        ]);

        let path = root_path.join("config_spec.toml");
        warpflow.save_toml(&path).unwrap();
        Ok(())
    }

    // 序列化测试
    #[test]
    fn software_save_load() -> SpecResult<()> {
        let root_path = PathBuf::from("./example/spec/redis");
        std::fs::create_dir_all(&root_path).owe_res()?;

        let conf_path = "./example/spec/redis/config_spec.toml";

        let artifact = Artifact::new(
            "redis-7.0.1",
            AfType::Bin,
            LocalAddr::from("redis-linux-7.tar.gz"),
        );
        let redis = SoftWare::new(
            artifact,
            WorkSpec::new("redis"),
            ConfSpecRef::new(conf_path),
        );

        let path = root_path.join("redis_7.toml");
        redis.save_toml(&path)?;

        let loaded = SoftWare::from_toml(&path)?;
        assert_eq!(loaded.workspec(), redis.workspec());
        Ok(())
    }
}
