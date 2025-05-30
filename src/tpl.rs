use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use fs_extra::dir::CopyOptions;
use handlebars::Handlebars;
use log::debug;
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, UvsResFrom, WithContext};
use serde::Serialize;

use crate::{error::SpecResult, module::setting::TemplatePath};

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
        setting: &TemplatePath,
    ) -> SpecResult<()> {
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
            Self::render_dir_impl(&handlebars, tpl, dst, &data, setting)
        } else {
            Self::render_file_impl(&handlebars, tpl, dst, &data, setting)
        }
    }
    fn render_dir_impl<T: Serialize>(
        handlebars: &Handlebars,
        tpl_dir: &PathBuf,
        dst: &PathBuf,
        data: &T,
        setting: &TemplatePath,
    ) -> SpecResult<()> {
        debug!("tpl dir: {}", tpl_dir.display());
        for entry in walkdir::WalkDir::new(tpl_dir) {
            let entry = entry.owe_data()?;
            let tpl_path = entry.path().to_path_buf();
            let relative_path = tpl_path.strip_prefix(tpl_dir).owe_data()?;
            let dst_path = Path::new(dst).join(relative_path);
            if setting.is_exclude(&tpl_path) {
                if let Some(dist) = dst_path.parent() {
                    fs_extra::copy_items(&[&tpl_path], &dist, &CopyOptions::default())
                        .owe_res()
                        .with(("tpl", &tpl_path))
                        .with(("dst", &dist.to_path_buf()))?;
                }
            } else {
                if tpl_path.is_dir() {
                    // 如果是目录，确保在目标位置创建对应的目录
                    std::fs::create_dir_all(&dst_path).owe_sys()?;
                    debug!("created dir: {}", dst_path.display());
                } else if tpl_path.is_file() {
                    // 如果是文件，则渲染模板
                    Self::render_file_impl(handlebars, &tpl_path, &dst_path, &data, setting)?;
                }
            }
            // 忽略其他类型（如符号链接等）
        }
        Ok(())
    }

    fn render_file_impl<T: Serialize>(
        handlebars: &Handlebars,
        tpl_path: &PathBuf,
        dst_path: &PathBuf,
        data: &T,
        setting: &TemplatePath,
    ) -> SpecResult<()> {
        debug!("tpl:{}", tpl_path.display());
        debug!("dst:{}", dst_path.display());

        let mut err_ctx = WithContext::want("render tpl");
        err_ctx.with("tpl", tpl_path.to_string_lossy());
        // 2. 验证模板文件
        if !tpl_path.exists() {
            return Err(StructError::from_conf("tpl path not exists".to_string()));
        }
        if setting.is_exclude(&tpl_path) {
            if let Some(dist) = dst_path.parent() {
                fs_extra::copy_items(&[&tpl_path], &dist, &CopyOptions::default())
                    .owe_res()
                    .with(("tpl", tpl_path))
                    .with(("dst", dist))?;
                return Ok(());
            }
            return Err(StructError::from_res("path not parent".into())).with(dst_path);
        }
        err_ctx.with("dst", dst_path.to_string_lossy());

        // 3. 准备目标文件
        let dst_path = Path::new(&dst_path);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).owe_sys()?;
        }
        if dst_path.exists() {
            std::fs::remove_file(dst_path).owe_sys()?;
        }

        // 4. 日志记录
        debug!(
            "Processing template: {} → {}",
            tpl_path.display(),
            dst_path.display()
        );

        // 5. 读取模板内容
        let template = std::fs::read_to_string(tpl_path)
            .owe_data()
            .with(&err_ctx)?;

        let mut dst_file = File::create(dst_path).owe_conf()?;

        handlebars
            .render_template_to_write(&template, data, &mut dst_file)
            .owe_biz()
            .with(&err_ctx)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o644); // rw-r--r--
            std::fs::set_permissions(dst_path, perms)
                .owe_sys()
                .with(&err_ctx)?;
        }
        println!(
            "render {:30} ---> {}",
            tpl_path.display(),
            dst_path.display()
        );

        debug!("Successfully generated: {}", dst_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::setting::TemplatePath;
    use tempfile::tempdir;

    #[test]
    fn test_render_path_with_handlebars() {
        // 准备测试目录结构
        let tmp_dir = tempdir().unwrap();
        let tpl_dir = tmp_dir.path().join("templates");
        std::fs::create_dir_all(&tpl_dir).unwrap();

        // 创建测试模板文件
        let tpl_file = tpl_dir.join("test.hbs");
        std::fs::write(&tpl_file, "Hello, {{name}}!").unwrap();

        // 创建测试数据文件
        let data_file = tmp_dir.path().join("data.json");
        std::fs::write(&data_file, r#"{"name": "World"}"#).unwrap();

        // 准备输出目录
        let output_dir = tmp_dir.path().join("output");

        // 空白的模板路径设置
        let setting = TemplatePath::default();

        // 执行渲染
        let result = TplRender::render_path(
            TPlEngineType::Handlebars,
            &tpl_file,
            &output_dir.join("output.txt"),
            &data_file,
            &setting,
        );

        assert!(result.is_ok());

        // 验证输出内容
        let output = std::fs::read_to_string(output_dir.join("output.txt")).unwrap();
        assert_eq!(output, "Hello, World!");
    }

    #[test]
    fn test_render_directory() {
        let tmp_dir = PathBuf::from("./temp/tpl");
        let tpl_dir = tmp_dir.join("templates");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).unwrap();
        }
        std::fs::create_dir_all(tpl_dir.join("subdir")).unwrap();

        // 创建多个模板文件
        std::fs::write(tpl_dir.join("main.hbs"), "Main: {{title}}").unwrap();
        std::fs::write(tpl_dir.join("subdir/file.hbs"), "Sub: {{title}}").unwrap();

        // 数据文件
        let data_file = tmp_dir.join("data.json");
        std::fs::write(&data_file, r#"{"title": "Test"}"#).unwrap();

        let output_dir = tmp_dir.join("output");
        let setting = TemplatePath::default();

        let result = TplRender::render_path(
            TPlEngineType::Handlebars,
            &tpl_dir,
            &output_dir,
            &data_file,
            &setting,
        );

        assert!(result.is_ok());
        assert_eq!(
            std::fs::read_to_string(output_dir.join("main.hbs")).unwrap(),
            "Main: Test"
        );
        assert_eq!(
            std::fs::read_to_string(output_dir.join("subdir/file.hbs")).unwrap(),
            "Sub: Test"
        );
    }

    #[test]
    fn test_excluded_files() {
        let tmp_dir = PathBuf::from("./temp/tpl2");
        let tpl_dir = tmp_dir.join("templates");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).unwrap();
        }
        std::fs::create_dir_all(&tpl_dir).unwrap();

        // 创建包含和不包含的文件
        std::fs::write(tpl_dir.join("render.hbs"), "{{content}}").unwrap();
        std::fs::write(tpl_dir.join("exclude.txt"), "raw content").unwrap();

        let data_file = tmp_dir.join("data.json");
        std::fs::write(&data_file, r#"{"content": "test"}"#).unwrap();

        let output_dir = tmp_dir.join("output");

        // 设置排除规则
        let mut setting = TemplatePath::default();
        setting.exclude_mut().push(tpl_dir.join("exclude.txt"));

        let _ = TplRender::render_path(
            TPlEngineType::Handlebars,
            &tpl_dir,
            &output_dir,
            &data_file,
            &setting,
        )
        .unwrap();

        // 验证模板文件被渲染
        assert_eq!(
            std::fs::read_to_string(output_dir.join("render.hbs")).unwrap(),
            "test"
        );
        // 验证排除文件被直接复制
        assert_eq!(
            std::fs::read_to_string(output_dir.join("exclude.txt")).unwrap(),
            "raw content"
        );
    }
}
