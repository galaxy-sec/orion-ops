use std::{collections::HashMap, path::PathBuf};

use derive_getters::Getters;
use derive_more::{Deref, From};
use serde_derive::{Deserialize, Serialize};

use crate::{error::SpecResult, types::Yamlable};

use super::{
    EnvDict,
    types::{EnvEvalable, ValueType},
};

pub type ValueMap = HashMap<String, ValueType>;

impl EnvEvalable<ValueMap> for ValueMap {
    fn env_eval(self, dict: &EnvDict) -> ValueMap {
        let mut vmap = HashMap::new();
        for (k, v) in self {
            let e_v = v.env_eval(dict);
            vmap.insert(k, e_v);
        }
        vmap
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default, From)]
#[serde(transparent)]
pub struct ValueDict {
    dict: HashMap<String, ValueType>,
}
impl ValueDict {
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, k: S, v: ValueType) -> Option<ValueType> {
        self.dict.insert(k.into(), v)
    }
    pub fn merge(&mut self, other: &ValueDict) {
        for (k, v) in other.iter() {
            if !self.contains_key(k) {
                self.dict.insert(k.clone(), v.clone());
            }
        }
    }
    pub fn env_eval(self, dict: &EnvDict) -> Self {
        let mut map = HashMap::new();
        for (k, v) in self.dict {
            let e_v = v.env_eval(dict);
            map.insert(k, e_v);
        }
        Self { dict: map }
    }
    pub fn eval_from_file(dict: &EnvDict, file_path: &PathBuf) -> SpecResult<Self> {
        let ins = ValueDict::from_yml(file_path)?;
        Ok(ins.env_eval(dict))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_toml_serialization() {
        let mut dict = ValueDict::new();
        dict.insert("key1".to_string(), ValueType::from("value1"));
        dict.insert("key2".to_string(), ValueType::from(42));
        let content = toml::to_string(&dict).unwrap();
        println!("{}", content);

        let loaded: ValueDict = toml::from_str(content.as_str()).unwrap();
        assert_eq!(dict, loaded);

        let content = serde_yaml::to_string(&dict).unwrap();
        println!("{}", content);

        let content = serde_json::to_string(&dict).unwrap();
        println!("{}", content);
    }
}
