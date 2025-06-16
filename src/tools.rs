use std::{
    fs,
    path::{Path, PathBuf},
};

use orion_error::{ErrorOwe, ErrorWith, UvsReason, UvsResFrom};
use url::Url;

use crate::error::{SpecReason, SpecResult, ToErr};

#[derive(Default, Clone, Debug)]
pub struct GitRepo {}
#[allow(dead_code)]
impl GitRepo {
    pub(crate) fn pull(_repo: &str) -> PathBuf {
        PathBuf::from("./os.sh")
    }
}
#[derive(Default, Clone, Debug)]
pub struct GxShell {}
#[allow(dead_code)]
impl GxShell {
    pub(crate) fn exec(_file: &str, _fun: &str) -> String {
        r#"{"result": true}"#.to_string()
    }
}
#[derive(Default, Clone, Debug)]
pub struct Http {}
impl Http {}
pub fn get_repo_name(url_str: &str) -> Option<String> {
    // 先尝试处理SSH格式的Git地址
    if url_str.starts_with("git@") {
        if let Some(repo_part) = url_str.split(':').next_back() {
            return repo_part.split('/').next_back().map(String::from);
        }
    }

    // 原有HTTP/HTTPS URL处理逻辑
    let url = Url::parse(url_str).ok()?;
    let last = url.path_segments()?.rev().find(|s| !s.is_empty());
    last.map(String::from)
}

pub fn get_sub_dirs(path: &Path) -> SpecResult<Vec<std::path::PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(path)
        .owe_res()
        .with(path)
        .want("read sub dirs")?
    {
        let entry = entry.owe_res()?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

pub fn test_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
pub fn make_clean_path(path: &Path) -> SpecResult<()> {
    if path.exists() {
        std::fs::remove_dir_all(path).owe_sys()?;
    }
    std::fs::create_dir_all(path).owe_sys()?;
    Ok(())
}
pub fn make_new_path(path: &Path) -> SpecResult<()> {
    if path.exists() {
        return SpecReason::Uvs(UvsReason::from_res("path exists".into())).err_result();
    }
    std::fs::create_dir_all(path).owe_sys()?;
    Ok(())
}

#[derive(Default)]
pub struct BoolFlag {
    is_suc: bool,
}
impl BoolFlag {
    pub fn flag_suc(&mut self) {
        self.is_suc = true;
    }
    pub fn is_suc(&self) -> bool {
        self.is_suc
    }
}
use std::env;

pub fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            // 跳过 '{'
            chars.next();

            // 收集变量名
            let mut var_name = String::new();
            let mut found_closing_brace = false;

            while let Some(c) = chars.next() {
                if c == '}' {
                    found_closing_brace = true;
                    break;
                }
                var_name.push(c);
            }

            // 处理变量替换
            if found_closing_brace {
                match env::var(&var_name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => {
                        result.push_str("${");
                        result.push_str(&var_name);
                        result.push('}');
                    }
                }
            } else {
                // 未闭合的花括号
                result.push_str("${");
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::tools::{expand_env_vars, get_repo_name};

    #[test]
    fn test_get_last_segment() {
        // 测试HTTP URL
        assert_eq!(
            get_repo_name("https://github.com/user/repo.git"),
            Some("repo.git".to_string())
        );

        // 测试HTTPS URL
        assert_eq!(
            get_repo_name("https://github.com/user/repo"),
            Some("repo".to_string())
        );

        // 测试SSH格式Git地址
        assert_eq!(
            get_repo_name("git@github.com:user/repo.git"),
            Some("repo.git".to_string())
        );

        // 测试SSH格式不带.git后缀
        assert_eq!(
            get_repo_name("git@gitlab.com:group/subgroup/repo"),
            Some("repo".to_string())
        );

        // 测试无效URL
        assert_eq!(get_repo_name("not_a_url"), None);
    }

    #[test]
    fn test_basic_expansion() {
        unsafe { env::set_var("HOME", "/home/user") };
        assert_eq!(expand_env_vars("${HOME}/bin"), "/home/user/bin");
    }

    #[test]
    fn test_multiple_variables() {
        unsafe { env::set_var("USER", "john") };
        unsafe { env::set_var("APP", "myapp") };
        assert_eq!(
            expand_env_vars("/opt/${APP}/bin/${USER}"),
            "/opt/myapp/bin/john"
        );
    }

    #[test]
    fn test_undefined_variable() {
        unsafe { env::remove_var("UNDEFINED_VAR") };
        assert_eq!(
            expand_env_vars("Path: ${UNDEFINED_VAR}/data"),
            "Path: ${UNDEFINED_VAR}/data"
        );
    }

    #[test]
    fn test_nested_braces() {
        unsafe { env::set_var("VAR", "value") };
        assert_eq!(expand_env_vars("${VAR}}"), "value}");
        assert_eq!(expand_env_vars("${VAR}}}"), "value}}");
    }

    #[test]
    fn test_unclosed_brace() {
        unsafe { env::set_var("HOME", "/home/user") };
        assert_eq!(expand_env_vars("${HOME"), "${HOME");
        assert_eq!(expand_env_vars("${HOME${USER"), "${HOME${USER");
    }

    #[test]
    fn test_empty_variable_name() {
        assert_eq!(expand_env_vars("${}"), "${}");
    }

    #[test]
    fn test_special_characters() {
        unsafe { env::set_var("VAR_WITH_UNDERSCORE", "ok") };
        assert_eq!(expand_env_vars("${VAR_WITH_UNDERSCORE}"), "ok");
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(expand_env_vars(""), "");
        assert_eq!(expand_env_vars("no variables"), "no variables");
        assert_eq!(expand_env_vars("$"), "$");
        assert_eq!(expand_env_vars("${"), "${");
        assert_eq!(expand_env_vars("}"), "}");
        assert_eq!(expand_env_vars("${}"), "${}");
    }

    #[test]
    fn test_consecutive_variables() {
        unsafe { env::set_var("A", "1") };
        unsafe { env::set_var("B", "2") };
        assert_eq!(expand_env_vars("${A}${B}"), "12");
    }
}
