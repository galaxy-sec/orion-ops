use std::collections::HashMap;

use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use super::{ValueDict, types::VarType};

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
//#[serde(transparent)]
pub struct VarCollection {
    vars: Vec<VarType>,
}
impl VarCollection {
    pub fn define(vars: Vec<VarType>) -> Self {
        Self { vars }
    }
    pub fn value_dict(&self) -> ValueDict {
        let mut dict = ValueDict::new();
        for var in &self.vars {
            dict.insert(var.name().to_string(), var.var_value()); // 可能需要 into() 转换
        }
        dict
    }
    // 基于VarType的name进行合并，相同的name会被覆盖
    pub fn merge(&self, other: &VarCollection) -> Self {
        let mut merged = HashMap::new();
        let mut order = Vec::new();

        // 先添加self的变量并记录顺序
        for var in &self.vars {
            let name = var.name().to_string();
            if !merged.contains_key(&name) {
                order.push(name.clone());
            }
            merged.insert(name, var.clone());
        }

        // 添加other的变量，同名会覆盖
        for var in &other.vars {
            let name = var.name().to_string();
            if !merged.contains_key(&name) {
                order.push(name.clone());
            }
            merged.insert(name, var.clone());
        }

        // 按原始顺序重新排序
        let mut result = Vec::new();
        for name in order {
            if let Some(var) = merged.get(&name) {
                result.push(var.clone());
            }
        }

        Self { vars: result }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_vars() {
        let vars1 = VarCollection::define(vec![
            VarType::from(("a", "1")),
            VarType::from(("b", true)),
            VarType::from(("c", 10)),
        ]);

        let vars2 = VarCollection::define(vec![
            VarType::from(("b", false)),
            VarType::from(("d", 3.14)),
        ]);

        let merged = vars1.merge(&vars2);

        // 验证合并后的变量数量
        assert_eq!(merged.vars().len(), 4);

        // 验证变量顺序
        let names: Vec<&str> = merged.vars().iter().map(|v| v.name()).collect();
        assert_eq!(names, vec!["a", "b", "c", "d"]);

        // 验证变量b被正确覆盖
        if let VarType::Bool(var) = &merged.vars()[1] {
            assert_eq!(var.var_value().value(), &false);
        } else {
            panic!("变量b类型错误");
        }
    }

    #[test]
    fn test_toml_serialization() {
        let collection = VarCollection::define(vec![
            VarType::from(("name", "Alice")),
            VarType::from(("age", 30)),
            VarType::from(("active", true)),
        ]);

        // 序列化为 TOML 字符串
        let toml_string = toml::to_string(&collection).expect("序列化失败");
        println!("{}", toml_string);

        // 反序列化测试
        let deserialized: VarCollection = toml::from_str(&toml_string).expect("反序列化失败");

        assert_eq!(collection, deserialized);
        assert_eq!(deserialized.vars().len(), 3);
        assert_eq!(deserialized.vars()[0].name(), "name");
        assert_eq!(deserialized.vars()[1].name(), "age");
        assert_eq!(deserialized.vars()[2].name(), "active");
    }
}
