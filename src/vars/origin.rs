use std::collections::HashMap;

use derive_getters::Getters;
use derive_more::Deref;
use serde_derive::{Deserialize, Serialize};

use super::{ValueDict, dict::ValueMap, types::ValueType};

pub type OriginMap = HashMap<String, OriginValue>;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OriginValue {
    origin: Option<String>,
    value: ValueType,
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default)]
pub struct OriginDict {
    dict: HashMap<String, OriginValue>,
}

impl From<ValueType> for OriginValue {
    fn from(value: ValueType) -> Self {
        Self {
            origin: None,
            value,
        }
    }
}

impl From<ValueDict> for OriginDict {
    fn from(value: ValueDict) -> Self {
        let mut dict = HashMap::new();
        for (k, v) in value.dict() {
            dict.insert(k.clone(), OriginValue::from(v.clone()));
        }
        Self { dict }
    }
}

impl OriginDict {
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
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
    pub fn export_origin(&self) -> OriginMap {
        let mut map = OriginMap::new();
        for (k, v) in &self.dict {
            map.insert(k.clone(), v.clone());
        }
        map
    }
}
