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
        std::fs::remove_dir_all(path).owe_sys().with(path)?;
    }
    std::fs::create_dir_all(path).owe_sys().with(path)?;
    Ok(())
}
pub fn ensure_path(path: &Path) -> SpecResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).owe_sys().with(path)?;
    }
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
