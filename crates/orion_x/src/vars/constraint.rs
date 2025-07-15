use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ValueScope {
    pub beg: u64,
    pub end: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ValueConstraint {
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "scope")]
    Scope(ValueScope),
}
impl ValueConstraint {
    pub fn scope(beg: u64, end: u64) -> Self {
        ValueConstraint::Scope(ValueScope { beg, end })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_value_constraint_serialization() {
        // 测试 Locked 变体的序列化
        let locked = ValueConstraint::Locked;
        let serialized = serde_json::to_string(&locked).unwrap();
        assert_eq!(serialized, r#""locked""#);

        // 测试 Scope 变体的序列化
        let scope = ValueConstraint::scope(1, 100);
        let serialized = serde_json::to_string(&scope).unwrap();
        assert_eq!(serialized, r#"{"scope":{"beg":1,"end":100}}"#);
    }

    #[test]
    fn test_value_constraint_deserialization() {
        // 测试 Locked 变体的反序列化
        let json = r#"{"locked":null}"#;
        let deserialized: ValueConstraint = serde_json::from_str(json).unwrap();
        assert!(matches!(deserialized, ValueConstraint::Locked));

        // 测试 Scope 变体的反序列化
        let json = r#"{"scope":{"beg":1, "end":100}}"#;
        let deserialized: ValueConstraint = serde_json::from_str(json).unwrap();
        let _constr = ValueConstraint::scope(5, 50);
        assert!(matches!(deserialized, _constr));
    }
}
