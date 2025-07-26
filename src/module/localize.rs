use std::path::{Path, PathBuf};

use fs_extra::dir::CopyOptions;
use log::{debug, info};
use orion_error::{ErrorOwe, ErrorWith, StructError, UvsConfFrom, UvsResFrom, WithContext};
use serde::Serialize;

use crate::{
    error::{ModReason, MainResult},
    module::setting::TemplatePath,
};
use orion_variate::tpl::{CommentFmt, CustTmplLabel, LabelCoverter, TplHandleBars};

use super::setting::TemplateConfig;

pub struct LocalizeTemplate<'a> {
    handlebars: TplHandleBars<'a>,
    cust_cover: CustTmplLabel,
}
impl Default for LocalizeTemplate<'_> {
    fn default() -> Self {
        Self {
            handlebars: TplHandleBars::init(),
            cust_cover: CustTmplLabel::None,
        }
    }
}
impl LocalizeTemplate<'_> {
    pub fn new(cust: TemplateConfig) -> Self {
        let convert = LabelCoverter::new(cust.origin().clone(), cust.target().clone());
        Self {
            handlebars: TplHandleBars::init(),
            cust_cover: CustTmplLabel::Setting(convert),
        }
    }
}
impl LocalizeTemplate<'_> {
    pub fn render_path(
        &self,
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &PathBuf,
        setting: &TemplatePath,
    ) -> MainResult<()> {
        let mut err_ctx = WithContext::want("render tpl path");
        // 处理目录模板
        err_ctx.with_path("data", data);
        let content = std::fs::read_to_string(data).owe_data().with(&err_ctx)?;
        err_ctx.with("need-fmt", "json");
        let data: serde_json::Value = serde_json::from_str(content.as_str())
            .owe_data()
            .with(&err_ctx)?;
        if tpl.is_dir() {
            self.render_dir_impl(tpl, dst, &data, setting)
                .with(&err_ctx)
        } else {
            self.render_file_impl(tpl, dst, &data, setting)
                .with(&err_ctx)
        }
    }

    fn render_dir_impl<T: Serialize>(
        &self,
        tpl_dir: &PathBuf,
        dst: &PathBuf,
        data: &T,
        setting: &TemplatePath,
    ) -> MainResult<()> {
        debug!("tpl dir: {}", tpl_dir.display());
        for entry in walkdir::WalkDir::new(tpl_dir) {
            let entry = entry.owe_data()?;
            let tpl_path = entry.path().to_path_buf();
            let relative_path = tpl_path.strip_prefix(tpl_dir).owe_data()?;
            let dst_path = Path::new(dst).join(relative_path);

            if tpl_path.is_dir() {
                // 如果是目录，确保在目标位置创建对应的目录
                std::fs::create_dir_all(&dst_path).owe_sys()?;
                debug!("created dir: {}", dst_path.display());
            } else if tpl_path.is_file() {
                // 如果是文件，则渲染模板
                self.render_file_impl(&tpl_path, &dst_path, &data, setting)?;
            }
        }
        Ok(())
    }

    fn render_file_impl<T: Serialize>(
        &self,
        tpl_path: &PathBuf,
        dst_path: &PathBuf,
        data: &T,
        templatize: &TemplatePath,
    ) -> MainResult<()> {
        debug!("tpl:{}", tpl_path.display());
        debug!("dst:{}", dst_path.display());

        let mut err_ctx = WithContext::want("render tpl");
        err_ctx.with("tpl", tpl_path.to_string_lossy());
        // 2. 验证模板文件
        if !tpl_path.exists() {
            return Err(StructError::from_conf("tpl path not exists".to_string())).with(&err_ctx);
        }
        if !templatize.is_include(tpl_path) {
            info!("ignore:{}", tpl_path.display());
            return Ok(());
        }
        if templatize.is_exclude(tpl_path) {
            if let Some(dist) = dst_path.parent() {
                println!("copy {:30} ---> {}", tpl_path.display(), dist.display());
                fs_extra::copy_items(&[&tpl_path], dist, &CopyOptions::default())
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

        //let convert = TplCoverter::new("[[", "]]", "{{", "}}", CommentLabel::yml_style());
        let template = self
            .cust_cover
            .convert(&CommentFmt::from(tpl_path.extension()), template)
            .with(&err_ctx)
            .owe(ModReason::Localize.into())?;
        //let mut dst_file = File::create(dst_path).owe_conf()?;

        let rendered_data = self
            .handlebars
            //.render_template_to_write(&template, data, &mut dst_file)
            .render_data(&template, data)
            .owe_biz()
            .with(&err_ctx)?;
        let completed = self
            .cust_cover
            .restore(rendered_data)
            .with(&err_ctx)
            .owe(ModReason::Localize.into())?;
        std::fs::write(dst_path, completed)
            .owe_conf()
            .with(dst_path)?;
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
    use orion_error::TestAssert;
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
        let result = LocalizeTemplate::default().render_path(
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
        let tmp_dir = tempdir().unwrap().path().to_path_buf();
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

        let result =
            LocalizeTemplate::default().render_path(&tpl_dir, &output_dir, &data_file, &setting);

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
        let tmp_dir = tempdir().unwrap().path().to_path_buf(); //PathBuf::from("./temp/tpl2");
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

        LocalizeTemplate::default()
            .render_path(&tpl_dir, &output_dir, &data_file, &setting)
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

    #[test]
    fn test_helm_nginx_rendering() {
        let root_dir = PathBuf::from("./test/helm");
        let helm_dir = PathBuf::from("./test/helm/nginx");
        let out_dir = PathBuf::from("./test/temp/nginx");
        if out_dir.exists() {
            std::fs::remove_dir_all(&out_dir).assert();
        }

        let mut setting = TemplatePath::default();
        setting.exclude_mut().push(helm_dir.join("templates"));

        let cust = TemplateConfig::example();

        //let _result = LocalizeTemplate::default()
        LocalizeTemplate::new(cust)
            .render_path(
                &helm_dir,
                &out_dir,
                &root_dir.join("value.json"), // 使用 values.yaml 作为数据源
                &setting,
            )
            .assert();
    }
}
