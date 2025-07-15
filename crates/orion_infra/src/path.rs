use derive_more::From;
use orion_error::{
    ErrorCode, ErrorOwe, ErrorWith, StructError, ToStructError, UvsReason, UvsResFrom,
};
use serde::Serialize;
use std::{fs, path::Path};
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum PathReason {
    #[error("brief {0}")]
    Brief(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for PathReason {
    fn error_code(&self) -> i32 {
        match self {
            PathReason::Brief(_) => 500,
            PathReason::Uvs(r) => r.error_code(),
        }
    }
}

pub type PathResult<T> = Result<T, StructError<PathReason>>;
pub type PathError = StructError<PathReason>;
pub fn make_clean_path(path: &Path) -> PathResult<()> {
    if path.exists() {
        std::fs::remove_dir_all(path).owe_sys().with(path)?;
    }
    std::fs::create_dir_all(path).owe_sys().with(path)?;
    Ok(())
}
pub fn ensure_path<P: AsRef<Path>>(path: P) -> PathResult<P> {
    if !path.as_ref().exists() {
        std::fs::create_dir_all(path.as_ref())
            .owe_sys()
            .with(path.as_ref())?;
    }
    Ok(path)
}

pub fn make_new_path(path: &Path) -> PathResult<()> {
    if path.exists() {
        return PathReason::Uvs(UvsReason::from_res("path exists".into())).err_result();
    }
    std::fs::create_dir_all(path).owe_sys()?;
    Ok(())
}

pub fn get_sub_dirs(path: &Path) -> PathResult<Vec<std::path::PathBuf>> {
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
