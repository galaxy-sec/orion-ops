use super::{VarValue, constraint::ValueConstraint, definition::VarDefinition};
use serde_derive::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VarType {
    #[serde(rename = "string")]
    String(VarDefinition<String>),
    #[serde(rename = "bool")]
    Bool(VarDefinition<bool>),
    #[serde(rename = "int")]
    Int(VarDefinition<u64>),
    #[serde(rename = "float")]
    Float(VarDefinition<f64>),
}
impl VarType {
    pub fn name(&self) -> &str {
        match self {
            VarType::String(var) => var.name(),
            VarType::Bool(var) => var.name(),
            VarType::Int(var) => var.name(),
            VarType::Float(var) => var.name(),
        }
    }
    pub fn constraint(mut self, constr: ValueConstraint) -> Self {
        match &mut self {
            VarType::String(var_define) => {
                var_define.set_constr(constr);
            }
            VarType::Bool(var_define) => {
                var_define.set_constr(constr);
            }
            VarType::Int(var_define) => {
                var_define.set_constr(constr);
            }
            VarType::Float(var_define) => {
                var_define.set_constr(constr);
            }
        }
        self
    }

    pub(crate) fn var_value(&self) -> ValueType {
        match self {
            VarType::String(var_define) => ValueType::String(var_define.var_value()),
            VarType::Bool(var_define) => ValueType::Bool(var_define.var_value()),
            VarType::Int(var_define) => ValueType::Int(var_define.var_value()),
            VarType::Float(var_define) => ValueType::Float(var_define.var_value()),
        }
    }
}

impl From<(&str, &str)> for VarType {
    fn from(value: (&str, &str)) -> Self {
        Self::String(VarDefinition::from(value))
    }
}
impl From<(&str, bool)> for VarType {
    fn from(value: (&str, bool)) -> Self {
        Self::Bool(VarDefinition::from(value))
    }
}
impl From<(&str, u64)> for VarType {
    fn from(value: (&str, u64)) -> Self {
        Self::Int(VarDefinition::from(value))
    }
}
impl From<(&str, f64)> for VarType {
    fn from(value: (&str, f64)) -> Self {
        Self::Float(VarDefinition::from(value))
    }
}

//#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    //#[serde(rename = "string")]
    String(VarValue<String>),
    //#[serde(rename = "bool")]
    Bool(VarValue<bool>),
    //#[serde(rename = "int")]
    Int(VarValue<u64>),
    //#[serde(rename = "float")]
    Float(VarValue<f64>),
}

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

impl From<&str> for ValueType {
    fn from(value: &str) -> Self {
        Self::String(VarValue::from(value))
    }
}
impl From<bool> for ValueType {
    fn from(value: bool) -> Self {
        Self::Bool(VarValue::from(value))
    }
}
impl From<u64> for ValueType {
    fn from(value: u64) -> Self {
        Self::Int(VarValue::from(value))
    }
}
impl From<f64> for ValueType {
    fn from(value: f64) -> Self {
        Self::Float(VarValue::from(value))
    }
}

#[cfg(test)]
mod tests {

    use crate::vars::constraint::ValueConstraint;

    use super::*;

    #[test]
    fn test_vartype_toml_serialization() {
        // 测试 String 类型的 TOML 序列化
        let string_var = VarType::from(("test_str", "hello")).constraint(ValueConstraint::Locked);
        let serialized = toml::to_string(&string_var).unwrap();
        let expected = r#"[string]
name = "test_str"
value = "hello"
constr = "locked"
"#;
        assert_eq!(serialized, expected);

        // 测试 Bool 类型的 TOML 序列化
        let bool_var = VarType::from(("test_bool", true)).constraint(ValueConstraint::scope(1, 10));
        let serialized = toml::to_string(&bool_var).unwrap();
        let expected = r#"[bool]
name = "test_bool"
value = true

[bool.constr.scope]
beg = 1
end = 10
"#;
        assert_eq!(serialized, expected);

        // 测试 Int 类型的 TOML 序列化
        let int_var = VarType::from(("test_int", 42));
        let serialized = toml::to_string(&int_var).unwrap();
        let expected = r#"[int]
name = "test_int"
value = 42
"#;
        assert_eq!(serialized, expected);

        // 测试 Float 类型的 TOML 序列化
        let float_var = VarType::from(("test_float", 3.14));
        let serialized = toml::to_string(&float_var).unwrap();
        let expected = r#"[float]
name = "test_float"
value = 3.14
"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_vartype_toml_deserialization() {
        // 测试 String 类型的 TOML 反序列化
        let toml_str = r#"
            [string]
            name = "test_str"
            value = "hello"
            constr = "locked"
        "#;
        let deserialized: VarType = toml::from_str(toml_str).unwrap();

        let _expect = VarType::from(("test_str", "hello")).constraint(ValueConstraint::Locked);
        assert!(matches!(deserialized, _expect));

        // 测试 Bool 类型的 TOML 反序列化
        let toml_str = r#"
            [bool]
            name = "test_bool"
            value = false

            [bool.constr.scope]
            beg = 5
            end = 50
        "#;
        let deserialized: VarType = toml::from_str(toml_str).unwrap();
        let _constr = ValueConstraint::scope(5, 50);
        let _expect = VarType::from(("test_bool", false)).constraint(_constr);
        assert!(matches!(deserialized, _expect));

        // 测试 Int 类型的 TOML 反序列化
        let toml_str = r#"
            [int]
            name = "test_int"
            value = 100
        "#;
        let deserialized: VarType = toml::from_str(toml_str).unwrap();
        let _expect = VarType::from(("test_int", 100));
        assert!(matches!(deserialized, _expect));

        // 测试 Float 类型的 TOML 反序列化
        let toml_str = r#"
            [float]
            name = "test_float"
            value = 1.618
        "#;
        let deserialized: VarType = toml::from_str(toml_str).unwrap();
        let _expect = VarType::from(("test_float", 1.618));
        assert!(matches!(deserialized, _expect));
    }
}
