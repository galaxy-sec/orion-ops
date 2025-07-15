use std::net::IpAddr;

use super::{env_eval::expand_env_vars, ValueDict};
use derive_more::Display;
use serde_derive::{Deserialize, Serialize};

pub type EnvDict = ValueDict;
pub trait EnvEvalable<T> {
    fn env_eval(self, dict: &EnvDict) -> T;
}

impl EnvEvalable<String> for String {
    fn env_eval(self, dict: &EnvDict) -> String {
        expand_env_vars(dict, self.as_str())
    }
}

impl EnvEvalable<Option<String>> for Option<String> {
    fn env_eval(self, dict: &EnvDict) -> Option<String> {
        self.map(|x| expand_env_vars(dict, x.as_str()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display)]
#[serde(untagged)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(u64),
    Float(f64),
    Ip(IpAddr),
}
impl EnvEvalable<ValueType> for ValueType {
    fn env_eval(self, dict: &EnvDict) -> ValueType {
        match self {
            ValueType::String(v) => ValueType::String(v.env_eval(dict)),
            _ => self,
        }
    }
}

impl From<&str> for ValueType {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<bool> for ValueType {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<u64> for ValueType {
    fn from(value: u64) -> Self {
        Self::Int(value)
    }
}
impl From<f64> for ValueType {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
