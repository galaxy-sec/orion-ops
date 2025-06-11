use std::{
    fs,
    path::{Path, PathBuf},
};

use orion_error::ErrorOwe;
use url::Url;

use crate::error::SpecResult;

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
    for entry in fs::read_dir(path).owe_res()? {
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
