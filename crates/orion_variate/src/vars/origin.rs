use derive_getters::Getters;
use derive_more::Deref;
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};

use super::{dict::ValueMap, types::ValueType, EnvDict, EnvEvalable, ValueDict};

pub type OriginMap = IndexMap<String, OriginValue>;

impl EnvEvalable<OriginMap> for OriginMap {
    fn env_eval(self, edict: &EnvDict) -> OriginMap {
        let mut origins = OriginMap::new();
        for (k, v) in self {
            let e_v = v.env_eval(edict);
            origins.insert(k, e_v);
        }
        origins
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OriginValue {
    origin: Option<String>,
    value: ValueType,
}

impl EnvEvalable<OriginValue> for OriginValue {
    fn env_eval(self, dict: &EnvDict) -> OriginValue {
        Self {
            origin: self.origin,
            value: self.value.env_eval(dict),
        }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default)]
pub struct OriginDict {
    dict: OriginMap,
}

impl From<ValueType> for OriginValue {
    fn from(value: ValueType) -> Self {
        Self {
            origin: None,
            value,
        }
    }
}
impl From<&str> for OriginValue {
    fn from(value: &str) -> Self {
        Self {
            origin: None,
            value: ValueType::from(value),
        }
    }
}
impl OriginValue {
    pub fn with_origin<S: Into<String>>(mut self, origin: S) -> Self {
        self.origin = Some(origin.into());
        self
    }
}

impl From<ValueDict> for OriginDict {
    fn from(value: ValueDict) -> Self {
        let mut dict = OriginMap::new();
        for (k, v) in value.dict() {
            dict.insert(k.clone(), OriginValue::from(v.clone()));
        }
        Self { dict }
    }
}

impl OriginDict {
    pub fn new() -> Self {
        Self {
            dict: OriginMap::new(),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, k: S, v: ValueType) -> Option<OriginValue> {
        self.dict.insert(k.into(), OriginValue::from(v))
    }
    pub fn set_source<S: Into<String> + Clone>(&mut self, lable: S) {
        for x in self.dict.values_mut() {
            if x.origin().is_none() {
                x.origin = Some(lable.clone().into());
            }
        }
    }
    pub fn merge(&mut self, other: &Self) {
        for (k, v) in other.iter() {
            if !self.contains_key(k) {
                self.dict.insert(k.clone(), v.clone());
            }
        }
    }
    pub fn export_value(&self) -> ValueMap {
        let mut map = ValueMap::new();
        for (k, v) in &self.dict {
            map.insert(k.clone(), v.value().clone());
        }
        map
    }
    pub fn export_dict(&self) -> ValueDict {
        ValueDict::from(self.export_value())
    }
    pub fn export_origin(&self) -> OriginMap {
        let mut map = OriginMap::new();
        for (k, v) in &self.dict {
            map.insert(k.clone(), v.clone());
        }
        map
    }
}
