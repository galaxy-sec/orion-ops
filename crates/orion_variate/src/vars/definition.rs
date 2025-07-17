use serde_derive::{Deserialize, Serialize};

use super::ValueType;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VarDefinition {
    name: String,
    desp: Option<String>,
    value: ValueType,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //constr: Option<ValueConstraint>,
}
impl VarDefinition {
    pub(crate) fn value(&self) -> ValueType {
        self.value.clone()
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }
}
impl From<(&str, &str)> for VarDefinition {
    fn from(value: (&str, &str)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
        }
    }
}
impl From<(&str, bool)> for VarDefinition {
    fn from(value: (&str, bool)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
        }
    }
}
impl From<(&str, u64)> for VarDefinition {
    fn from(value: (&str, u64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
        }
    }
}
impl From<(&str, f64)> for VarDefinition {
    fn from(value: (&str, f64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
        }
    }
}

impl From<(&str, ValueType)> for VarDefinition {
    fn from(value: (&str, ValueType)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: value.1,
        }
    }
}
