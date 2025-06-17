use std::collections::HashMap;

use derive_getters::Getters;
use derive_more::Deref;
use serde_derive::{Deserialize, Serialize};

use super::types::{EnvEvalable, ValueType};

pub type ValueMap = HashMap<String, ValueType>;

impl EnvEvalable<ValueMap> for ValueMap {
    fn env_eval(self) -> ValueMap {
        let mut dict = HashMap::new();
        for (k, v) in self {
            let e_v = v.env_eval();
            dict.insert(k, e_v);
        }
        dict
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default)]
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
    pub fn env_eval(self) -> Self {
        let mut dict = HashMap::new();
        for (k, v) in self.dict {
            let e_v = v.env_eval();
            dict.insert(k, e_v);
        }
        Self { dict }
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
