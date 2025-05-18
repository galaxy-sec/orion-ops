use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use handlebars::Handlebars;
use log::debug;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, UvsReason, WithContext};
use serde::Serialize;

use crate::error::RunResult;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TPlEngineType {
    #[default]
    Handlebars,
    Helm,
}
impl FromStr for TPlEngineType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "handlebars" => Ok(Self::Handlebars),
            "helm" => Ok(Self::Helm),
            _ => Err("unknow engine".to_string()),
        }
    }
}
impl Display for TPlEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TPlEngineType::Handlebars => "handlebars",
            TPlEngineType::Helm => "helm",
        };
        write!(f, "{}", msg)
    }
}
pub struct TplRender;
impl TplRender {
    pub fn render_path(
        engine: TPlEngineType,
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &PathBuf,
    ) -> RunResult<()> {
        let mut err_ctx = WithContext::want("render tpl path");
        // 处理目录模板
        if engine != TPlEngineType::Handlebars {
            unimplemented!("not support other engine")
        }
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        // 准备数据

        let content = std::fs::read_to_string(data).owe_data().with(&err_ctx)?;
        err_ctx.with("need-fmt", "json");
        let data: serde_json::Value = serde_json::from_str(content.as_str())
            .owe_data()
            .with(&err_ctx)?;
        if tpl.is_dir() {
            Self::render_dir_impl(&handlebars, &tpl, &dst, &data)
        } else {
            Self::render_file_impl(&handlebars, &tpl, &dst, &data)
        }
    }
    fn render_dir_impl<T: Serialize>(
        handlebars: &Handlebars,
        tpl_dir: &PathBuf,
        dst: &PathBuf,
        data: &T,
    ) -> RunResult<()> {
        debug!("tpl dir: {}", tpl_dir.display());
        for entry in walkdir::WalkDir::new(tpl_dir) {
            let entry = entry.owe_data()?;
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(tpl_dir).owe_data()?;
            let dst_path = Path::new(dst).join(relative_path);

            if entry_path.is_dir() {
                // 如果是目录，确保在目标位置创建对应的目录
                std::fs::create_dir_all(&dst_path).owe_sys()?;
                debug!("created dir: {}", dst_path.display());
            } else if entry_path.is_file() {
                // 如果是文件，则渲染模板
                Self::render_file_impl(handlebars, &entry_path.to_path_buf(), &dst_path, &data)?;
            }
            // 忽略其他类型（如符号链接等）
        }
        Ok(())
    }

    fn render_file_impl<T: Serialize>(
        handlebars: &Handlebars,
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &T,
    ) -> RunResult<()> {
        debug!("tpl:{}", tpl.display());
        debug!("dst:{}", dst.display());

        let mut err_ctx = WithContext::want("render tpl");
        err_ctx.with("tpl", tpl.to_string_lossy());
        // 2. 验证模板文件
        let tpl_path = Path::new(&tpl);
        if !tpl_path.exists() {
            return Err(StructError::from_uvs_rs(UvsReason::from_conf(
                "tpl path not exists".to_string(),
            )));
        }
        err_ctx.with("dst", dst.to_string_lossy());
        // 3. 准备目标文件
        let dst_path = Path::new(&dst);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).owe_sys()?;
        }
        if dst_path.exists() {
            std::fs::remove_file(dst).owe_sys()?;
        }

        // 4. 日志记录
        debug!("Processing template: {} → {}", tpl.display(), dst.display());

        // 5. 读取模板内容
        let template = std::fs::read_to_string(tpl).owe_data().with(&err_ctx)?;

        let mut dst_file = File::create(dst).owe_conf()?;

        handlebars
            .render_template_to_write(&template, data, &mut dst_file)
            .owe_biz()
            .with(&err_ctx)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o644); // rw-r--r--
            std::fs::set_permissions(dst, perms)
                .owe_sys()
                .with(&err_ctx)?;
        }
        println!("render {:30} ---> {}", tpl.display(), dst.display());

        debug!("Successfully generated: {}", dst.display());
        Ok(())
    }
}
