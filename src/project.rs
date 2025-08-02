use std::path::{Path, PathBuf};

use log::info;
use orion_common::serde::{Configable, ValueConfable, Yamlable};
use orion_error::ErrorOwe;
use orion_infra::path::ensure_path;
use orion_variate::vars::{EnvDict, EnvEvalable, OriginDict, ValueDict, ValueType, VarCollection};

use crate::{
    const_vars::{VALUE_DIR, VALUE_FILE},
    error::MainResult,
    module::model::TargetValuePaths,
    types::LocalizeOptions,
};

pub fn load_project_global_value(root: &Path, options: &Option<String>) -> MainResult<ValueDict> {
    let value_root = ensure_path(root.join(VALUE_DIR)).owe_logic()?;
    let value_file = if let Some(v_file) = options {
        PathBuf::from(v_file)
    } else {
        let v_file = value_root.join(VALUE_FILE);
        if !v_file.exists() {
            let mut dict = ValueDict::new();
            dict.insert("SAMPLE_KEY", ValueType::from("SAMPLE_VAL"));
            dict.save_valconf(&v_file).owe_res()?;
        }
        v_file
    };
    let dict = ValueDict::from_yml(&value_file).owe_logic()?;
    Ok(dict)
}

pub fn mix_used_value(
    options: LocalizeOptions,
    value_paths: &TargetValuePaths,
    vars: &VarCollection,
) -> MainResult<OriginDict> {
    let mut used = OriginDict::from(options.raw_value().clone().env_eval(&EnvDict::default()));
    used.set_source("global");
    if value_paths.user_value_file().exists() && !options.use_default_value() {
        let user_dict = ValueDict::from_conf(value_paths.user_value_file()).owe_res()?;
        let mut user_dict = OriginDict::from(user_dict.env_eval(&used.export_dict()));
        user_dict.set_source("mod-cust");
        used.merge(&user_dict);
        info!(target:"mod/target", "use  model value : {}", value_paths.user_value_file().display());
    }
    let mut default_dict = OriginDict::from(vars.value_dict().env_eval(&used.export_dict()));
    default_dict.set_source("mod-default");
    used.merge(&default_dict);
    Ok(used)
}

#[cfg(test)]
mod tests {
    use crate::const_vars::USER_VALUE_FILE;

    use super::*;
    use orion_variate::vars::{OriginValue, VarDefinition};
    use tempfile::tempdir;

    fn test_init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_build_used_value_with_default_only() {
        test_init();
        let vars = VarCollection::define(vec![VarDefinition::from(("TEST_KEY", "default_value"))]);
        let options = LocalizeOptions::new(ValueDict::new(), false);
        let temp_dir = tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("default_value").with_origin("mod-default"))
        );
    }

    #[test]
    fn test_build_used_value_with_global_value() {
        test_init();
        let mut global_dict = ValueDict::new();
        global_dict.insert("TEST_KEY".to_string(), ValueType::from("global_value"));
        global_dict.insert("PRJ_SPACE".to_string(), ValueType::from("galaxy"));
        let vars = VarCollection::define(vec![
            VarDefinition::from(("TEST_KEY", "default_value")),
            VarDefinition::from(("PRJ_SPACE", "${HOME}")),
            VarDefinition::from(("SVR_NAME", "gflow")),
            VarDefinition::from(("MOD_SPACE", "${PRJ_SPACE}/${SVR_NAME}")),
            VarDefinition::from(("SVR_SPACE", "/home/${SVR_NAME}")),
        ]);
        let options = LocalizeOptions::new(global_dict, false);
        let temp_dir = tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("global_value").with_origin("global"))
        );
        assert_eq!(
            result.get("PRJ_SPACE"),
            Some(&OriginValue::from("galaxy").with_origin("global"))
        );
        assert_eq!(
            result.get("SVR_SPACE"),
            Some(&OriginValue::from("/home/gflow").with_origin("mod-default"))
        );
        assert_eq!(
            result.get("MOD_SPACE"),
            Some(&OriginValue::from("galaxy/gflow").with_origin("mod-default"))
        );
    }

    #[test]
    fn test_build_used_value_with_user_value() {
        test_init();
        let temp_dir = tempdir().unwrap();
        let user_value_path = temp_dir.path().join(USER_VALUE_FILE);
        std::fs::write(&user_value_path, "TEST_KEY: user_value").unwrap();

        let vars = VarCollection::define(vec![VarDefinition::from(("TEST_KEY", "default_value"))]);
        let options = LocalizeOptions::new(ValueDict::new(), false);
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("user_value").with_origin("mod-cust"))
        );
    }

    #[test]
    fn test_build_used_value_merge_precedence() {
        test_init();
        let temp_dir = tempdir().unwrap();
        let cust_value_path = temp_dir.path().join(USER_VALUE_FILE);
        std::fs::write(
            &cust_value_path,
            "TEST_KEY: user_value\nUSER_ONLY: user_only",
        )
        .unwrap();

        let mut global_dict = ValueDict::new();
        global_dict.insert("TEST_KEY".to_string(), ValueType::from("global_value"));
        global_dict.insert("GLOBAL_ONLY".to_string(), ValueType::from("global_only"));

        let vars = VarCollection::define(vec![
            VarDefinition::from(("TEST_KEY", "default_value")),
            VarDefinition::from(("DEFAULT_ONLY", "default_only")),
        ]);
        let options = LocalizeOptions::new(global_dict, false);
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        // 验证优先级: global > cust  > default
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("global_value").with_origin("global"))
        );
        // 验证各层特有键都存在
        assert_eq!(
            result.get("GLOBAL_ONLY"),
            Some(&OriginValue::from("global_only").with_origin("global"))
        );
        assert_eq!(
            result.get("USER_ONLY"),
            Some(&OriginValue::from("user_only").with_origin("mod-cust"))
        );
        assert_eq!(
            result.get("DEFAULT_ONLY"),
            Some(&OriginValue::from("default_only").with_origin("mod-default"))
        );
    }

    #[test]
    fn test_empty_vars_returns_empty_dict() {
        test_init();
        let vars = VarCollection::define(vec![]);
        let options = LocalizeOptions::new(ValueDict::new(), false);
        let temp_dir = tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_complex_value_types() {
        test_init();

        let vars = VarCollection::define(vec![
            VarDefinition::from(("STRING_VAR", ValueType::from("default_string"))),
            VarDefinition::from(("NUMBER_VAR", ValueType::from(42))),
            VarDefinition::from(("BOOL_VAR", ValueType::from(true))),
        ]);
        let options = LocalizeOptions::new(ValueDict::new(), false);
        let temp_dir = tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();

        assert_eq!(
            result.get("STRING_VAR"),
            Some(&OriginValue::from(ValueType::from("default_string")).with_origin("mod-default"))
        );
        assert_eq!(
            result.get("NUMBER_VAR"),
            Some(&OriginValue::from(ValueType::from(42)).with_origin("mod-default"))
        );
        assert_eq!(
            result.get("BOOL_VAR"),
            Some(&OriginValue::from(ValueType::from(true)).with_origin("mod-default"))
        );
    }

    #[test]
    fn test_env_variable_substitution() {
        test_init();
        unsafe {
            std::env::set_var("TEST_ENV_VAR", "substituted_value");
        }

        let vars = VarCollection::define(vec![
            VarDefinition::from(("ENV_VAR", "${TEST_ENV_VAR}")),
            VarDefinition::from(("MIXED_VAR", "prefix_${TEST_ENV_VAR}_suffix")),
        ]);
        let options = LocalizeOptions::new(ValueDict::new(), false);
        let temp_dir = tempdir().unwrap();
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();

        assert_eq!(
            result.get("ENV_VAR"),
            Some(&OriginValue::from("substituted_value").with_origin("mod-default"))
        );
        assert_eq!(
            result.get("MIXED_VAR"),
            Some(&OriginValue::from("prefix_substituted_value_suffix").with_origin("mod-default"))
        );

        unsafe {
            std::env::remove_var("TEST_ENV_VAR");
        }
    }

    #[test]
    fn test_use_default_value_flag() {
        test_init();
        let temp_dir = tempdir().unwrap();
        let user_value_path = temp_dir.path().join(USER_VALUE_FILE);
        std::fs::write(&user_value_path, "TEST_KEY: user_value").unwrap();

        let vars = VarCollection::define(vec![VarDefinition::from(("TEST_KEY", "default_value"))]);
        let options = LocalizeOptions::new(ValueDict::new(), true);
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("default_value").with_origin("mod-default"))
        );
    }

    #[test]
    fn test_global_value_override_precedence() {
        test_init();
        let temp_dir = tempdir().unwrap();
        let user_value_path = temp_dir.path().join(USER_VALUE_FILE);
        std::fs::write(&user_value_path, "TEST_KEY: user_value").unwrap();

        let mut global_dict = ValueDict::new();
        global_dict.insert("TEST_KEY".to_string(), ValueType::from("global_value"));

        let vars = VarCollection::define(vec![VarDefinition::from(("TEST_KEY", "default_value"))]);
        let options = LocalizeOptions::new(global_dict, false);
        let value_paths = TargetValuePaths::from(&temp_dir.path().to_path_buf());

        let result = mix_used_value(options, &value_paths, &vars).unwrap();
        // 全局值应该覆盖用户值和默认值
        assert_eq!(
            result.get("TEST_KEY"),
            Some(&OriginValue::from("global_value").with_origin("global"))
        );
    }
}
