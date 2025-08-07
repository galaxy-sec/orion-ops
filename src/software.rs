use crate::{conf::ConfSpecRef, spec::WorkSpec};
use derive_getters::Getters;
use orion_variate::ext::Artifact;
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
    #[allow(dead_code)]
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
#[serde(untagged)]
pub enum Constraint {
    Matching(String),
    WithInScope(u64, u64),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use orion_common::serde::Configable;
    use orion_error::ErrorOwe;
    use orion_variate::addr::LocalPath;
    use tempfile::env::temp_dir;

    use crate::{
        conf::{ConfFile, ConfSpec},
        const_vars::CONFS_DIR,
        error::MainResult,
    };

    use super::*;

    // 基础功能测试
    #[test]
    fn test_conf_spec_creation() {
        let spec = ConfSpec::new("1.0", CONFS_DIR);
        assert_eq!(spec.version(), "1.0");
    }

    // 序列化测试
    #[test]
    fn confspec_save_load() -> MainResult<()> {
        let root_path = PathBuf::from("./example/spec/redis");
        std::fs::create_dir_all(&root_path).owe_res()?;
        let mut redis = ConfSpec::new("1.0", CONFS_DIR);
        redis.add(ConfFile::new("./nginx.conf"));

        let path = root_path.join("config_spec.yml");
        redis.save_conf(&path).unwrap();
        let loaded = ConfSpec::from_conf(&path).unwrap();
        assert_eq!(redis.version(), loaded.version());

        let warpflow = ConfSpec::default_from_files(vec![
            "./conf/dvron.toml",
            "./conf/dvgen.toml",
            "./sink/framework.toml",
        ]);

        let path = root_path.join("config_spec.yml");
        warpflow.save_conf(&path).unwrap();
        Ok(())
    }

    // 序列化测试
    #[test]
    fn software_save_load() -> MainResult<()> {
        //let root_path = PathBuf::from("./example/spec/redis");
        let root_path = temp_dir();
        std::fs::create_dir_all(&root_path).owe_res()?;

        let conf_path = "./example/spec/redis/config_spec.yml";

        let artifact = Artifact::new(
            "redis-7.0.1",
            "7.0.1",
            LocalPath::from("redis-linux-7.tar.gz"),
            "redis-linux-7.tar.gz",
        );
        let redis = SoftWare::new(
            artifact,
            WorkSpec::new("redis"),
            ConfSpecRef::new(conf_path)?,
        );

        let path = root_path.join("redis_7.yml");
        redis.save_conf(&path).owe_logic()?;

        let loaded = SoftWare::from_conf(&path).owe_logic()?;
        assert_eq!(loaded.workspec(), redis.workspec());
        Ok(())
    }
}
