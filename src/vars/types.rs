use super::{ValueDict, env_eval::expand_env_vars};
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
//#[derive(Clone, Debug, PartialEq, Display, From)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(u64),
    Float(f64),
}
impl EnvEvalable<ValueType> for ValueType {
    fn env_eval(self, dict: &EnvDict) -> ValueType {
        match self {
            ValueType::String(v) => ValueType::String(v.env_eval(dict)),
            _ => self,
        }
    }
}

/*
impl serde::Serialize for ValueType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ValueType::String(v) => v.serialize(serializer),
            ValueType::Bool(v) => v.serialize(serializer),
            ValueType::Int(v) => v.serialize(serializer),
            ValueType::Float(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ValueType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 使用 serde::DeserializeSeed 来避免多次使用 deserializer
        struct ValueTypeVisitor;

        impl serde::de::Visitor<'_> for ValueTypeVisitor {
            type Value = ValueType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string, bool, integer or float value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(ValueType::Bool(VarValue::from(v)))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(ValueType::Int(VarValue::from(v as u64)))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(ValueType::Int(VarValue::from(v)))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(ValueType::Float(VarValue::from(v)))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueType::String(VarValue::from(v)))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
                Ok(ValueType::String(VarValue::from(v)))
            }
        }

        deserializer.deserialize_any(ValueTypeVisitor)
    }
}
    */

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
