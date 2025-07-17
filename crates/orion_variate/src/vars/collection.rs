use std::path::Path;

use derive_getters::Getters;
use indexmap::IndexMap;
use orion_error::ErrorOwe;
use serde_derive::{Deserialize, Serialize};

use orion_common::serde::Yamlable;

use super::{EnvDict, EnvEvalable, ValueDict, VarDefinition, error::VarsResult};

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
//#[serde(transparent)]
pub struct VarCollection {
    vars: Vec<VarDefinition>,
}
impl VarCollection {
    pub fn define(vars: Vec<VarDefinition>) -> Self {
        Self { vars }
    }
    pub fn value_dict(&self) -> ValueDict {
        let mut dict = ValueDict::new();
        for var in &self.vars {
            dict.insert(var.name().to_string(), var.value()); // 可能需要 into() 转换
        }
        dict
    }
    // 基于VarType的name进行合并，相同的name会被覆盖
    pub fn merge(&self, other: VarCollection) -> Self {
        let mut merged = IndexMap::new();
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
        for var in other.vars {
            let name = var.name().to_string();
            if !merged.contains_key(&name) {
                order.push(name.clone());
            }
            merged.insert(name, var);
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
    pub fn eval_from_file(dict: &EnvDict, file_path: &Path) -> VarsResult<Self> {
        let mut cur_dict = dict.clone();
        let ins = VarCollection::from_yml(file_path).owe_res()?;
        Ok(ins.eval_import(&mut cur_dict))
    }

    fn eval_import(self, dict: &mut ValueDict) -> Self {
        let mut vars = Vec::new();
        for v in self.vars {
            let e_v = v.value().env_eval(dict);
            dict.insert(v.name(), e_v.clone());
            vars.push(VarDefinition::from((v.name(), e_v)));
        }
        Self { vars }
    }
}

#[cfg(test)]
mod tests {
    use crate::vars::ValueType;

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // 辅助函数：创建临时测试文件
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }
    #[test]
    fn test_eval_from_file_basic() {
        // 准备测试数据
        let file = create_temp_file(
            r#"
        vars:
          - name: "username"
            value: "admin"
          - name: "account"
            value: ${username}
        "#,
        );

        // 准备环境字典
        let env_dict = EnvDict::new();

        // 执行方法
        let result = VarCollection::eval_from_file(&env_dict, file.path()).unwrap();

        // 验证结果
        assert_eq!(result.vars().len(), 2);
        assert_eq!(result.vars()[0].name(), "username");
        assert_eq!(result.vars()[0].value(), ValueType::from("admin"));
        assert_eq!(result.vars()[1].name(), "account");
        assert_eq!(result.vars()[1].value(), ValueType::from("admin"));
    }

    #[test]
    fn test_merge_vars() {
        let vars1 = VarCollection::define(vec![
            VarDefinition::from(("a", "1")),
            VarDefinition::from(("b", true)),
            VarDefinition::from(("c", 10)),
        ]);

        let vars2 = VarCollection::define(vec![
            VarDefinition::from(("b", false)),
            VarDefinition::from(("d", 3.33)),
        ]);

        let merged = vars1.merge(vars2);

        // 验证合并后的变量数量
        assert_eq!(merged.vars().len(), 4);

        // 验证变量顺序
        let names: Vec<&str> = merged.vars().iter().map(|v| v.name()).collect();
        assert_eq!(names, vec!["a", "b", "c", "d"]);

        // 验证变量b被正确覆盖
        if let ValueType::Bool(var) = &merged.vars()[1].value() {
            assert_eq!(var, &false);
        } else {
            panic!("变量b类型错误");
        }
    }

    #[test]
    fn test_toml_serialization() {
        let collection = VarCollection::define(vec![
            VarDefinition::from(("name", "Alice")),
            VarDefinition::from(("age", 30)),
            VarDefinition::from(("active", true)),
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
