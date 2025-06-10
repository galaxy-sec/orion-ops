use std::collections::HashMap;

use derive_getters::Getters;
use derive_more::Deref;
use serde_derive::{Deserialize, Serialize};

use super::types::ValueType;

pub type ValueMap = HashMap<String, ValueType>;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DictItem {
    #[serde(skip, default)]
    source: Option<String>,
    value: ValueType,
}
impl From<ValueType> for DictItem {
    fn from(value: ValueType) -> Self {
        Self {
            source: None,
            value,
        }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default)]
#[serde(transparent)]
pub struct ValueDict {
    dict: HashMap<String, DictItem>,
}
impl ValueDict {
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, k: S, v: ValueType) -> Option<DictItem> {
        self.dict.insert(k.into(), DictItem::from(v))
    }
    pub fn set_source<S: Into<String> + Clone>(&mut self, lable: S) {
        for x in self.dict.values_mut() {
            if x.source().is_none() {
                x.source = Some(lable.clone().into());
            }
        }
    }
    pub fn merge(&mut self, other: &ValueDict) {
        for (k, v) in other.iter() {
            if !self.contains_key(k) {
                self.dict.insert(k.clone(), v.clone());
            }
        }
    }
    pub fn export(&self) -> ValueMap {
        let mut map = ValueMap::new();
        for (k, v) in &self.dict {
            map.insert(k.clone(), v.value().clone());
        }
        map
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
