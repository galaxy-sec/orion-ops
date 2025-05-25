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
pub fn get_last_segment(url_str: &str) -> Option<String> {
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
