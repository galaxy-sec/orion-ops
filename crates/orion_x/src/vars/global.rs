use std::{
    env::{self, current_dir},
    path::PathBuf,
};

use log::info;
use orion_error::{ErrorOwe, ErrorWith};

use super::error::VarsResult;

pub fn setup_start_env_vars() -> VarsResult<()> {
    unsafe { std::env::set_var("GXL_OS_SYS", format_os_sys().as_str()) };
    let start_root = current_dir().owe_sys().want("get current dir")?;
    unsafe { std::env::set_var("GXL_START_ROOT", start_root.display().to_string()) };
    let prj_root = find_project_define().unwrap_or(PathBuf::from("UNDEFIN"));
    unsafe { std::env::set_var("GXL_PRJ_ROOT", format!("{}", prj_root.display())) };
    Ok(())
}

fn get_os_info() -> (String, String, u64) {
    let info = os_info::get();
    let os_type = match info.os_type() {
        os_info::Type::Macos => "macos".to_string(),
        _ => info.os_type().to_string().to_lowercase(),
    };

    let arch = info.architecture().unwrap_or("unknown").to_string();
    let ver_major = match info.version() {
        os_info::Version::Semantic(major, _, _) => *major,
        _ => 0,
    };

    (arch, os_type, ver_major)
}

fn format_os_sys() -> String {
    let (arch, os_type, ver_major) = get_os_info();
    format!("{}_{}_{}", arch, os_type, ver_major)
}

/// 从当前目录开始向上查找 _gal/project.toml 文件
/// 如果找到则返回其绝对路径的PathBuf，未找到则返回None
pub fn find_project_define() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    loop {
        let project_file = current_dir.join("_gal").join("project.toml");
        if project_file.exists() {
            //let project_root = current_dir.clone();
            return Some(current_dir);
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // 已到达根目录
        }
    }

    None
}
pub struct WorkDir {
    original_dir: PathBuf,
}

impl WorkDir {
    #[allow(dead_code)]
    pub fn change<S: Into<PathBuf>>(target_dir: S) -> std::io::Result<Self> {
        let original_dir = env::current_dir()?;
        let target = target_dir.into();
        info!("set current dir:{}", target.display());
        env::set_current_dir(target)?;
        Ok(Self { original_dir })
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        info!("set current dir:{}", self.original_dir.display());
        if let Err(e) = env::set_current_dir(&self.original_dir) {
            log::error!("Failed to restore directory: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {

    use tempfile::TempDir;

    use crate::vars::global::{find_project_define, WorkDir};

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_in_current_dir() {
        // 创建临时目录
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 设置当前工作目录为临时目录
        //env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");
        let _wd = WorkDir::change(temp_dir.path());

        // 调用函数并断言结果
        assert!(find_project_define().is_some())
    }

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_in_parent_dir() {
        // 创建临时目录结构: temp_dir/child/_gal/project.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let child_dir = temp_dir.path().join("child");
        std::fs::create_dir(&child_dir).expect("Failed to create child dir");
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 设置当前工作目录为child_dir
        let _wd = WorkDir::change(&child_dir);
        //env::set_current_dir(&child_dir).expect("Failed to set current dir");

        // 调用函数应找到父目录中的文件
        assert!(find_project_define().is_some());
    }

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_not_found() {
        // 创建临时目录，不创建_gal/project.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let _wd = WorkDir::change(temp_dir.path());
        // 设置当前工作目录为临时目录
        //env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");

        // 调用函数应返回None
        assert_eq!(find_project_define(), None);
    }
}
