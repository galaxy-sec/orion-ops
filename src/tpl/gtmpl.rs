use std::path::PathBuf;

use fs_extra::dir::CopyOptions;

use orion_error::{ErrorOwe, ErrorWith, StructError, UvsResFrom, WithContext};

use crate::{error::SpecResult, module::setting::TemplatePath};

pub struct TplGtmpl;
impl TplGtmpl {
    pub fn render_path(
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &PathBuf,
        setting: &TemplatePath,
    ) -> SpecResult<()> {
        let mut err_ctx = WithContext::want("render gtmpl path");

        // 读取模板数据
        err_ctx.with_path("data", data);
        let content = std::fs::read_to_string(data).owe_data().with(&err_ctx)?;
        let data: serde_json::Value = serde_json::from_str(&content).owe_data().with(&err_ctx)?;

        if tpl.is_dir() {
            Self::render_dir_gtmpl(tpl, dst, &data, setting)
        } else {
            Self::render_file_gtmpl(tpl, dst, &data, setting)
        }
    }

    fn render_dir_gtmpl(
        tpl_dir: &PathBuf,
        dst: &PathBuf,
        data: &serde_json::Value,
        setting: &TemplatePath,
    ) -> SpecResult<()> {
        for entry in walkdir::WalkDir::new(tpl_dir) {
            let entry = entry.owe_data()?;
            let tpl_path = entry.path().to_path_buf();
            let relative_path = tpl_path.strip_prefix(tpl_dir).owe_data()?;
            let dst_path = dst.join(relative_path);
            if tpl_path.is_dir() {
                std::fs::create_dir_all(&dst_path).owe_sys()?;
            } else {
                Self::render_file_gtmpl(&tpl_path, &dst_path, data, setting)?;
            }
        }
        Ok(())
    }

    fn render_file_gtmpl(
        tpl_path: &PathBuf,
        dst_path: &PathBuf,
        data: &serde_json::Value,
        setting: &TemplatePath,
    ) -> SpecResult<()> {
        if setting.is_exclude(&tpl_path) {
            if let Some(dist) = dst_path.parent() {
                println!("copy {:30} ---> {}", tpl_path.display(), dist.display());
                fs_extra::copy_items(&[&tpl_path], &dist, &CopyOptions::default())
                    .owe_res()
                    .with(("tpl", tpl_path))
                    .with(("dst", dist))?;

                return Ok(());
            }
            return Err(StructError::from_res("path not parent".into())).with(dst_path);
        }

        let template = std::fs::read_to_string(tpl_path).owe_data()?;
        let gtmpl_data = json_to_gtmpl(data);
        let rendered = gtmpl::template(&template, gtmpl_data)
            .owe_biz()
            .with(tpl_path)?;

        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).owe_sys()?;
        }
        std::fs::write(dst_path, rendered).owe_sys()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dst_path, std::fs::Permissions::from_mode(0o644)).owe_sys()?;
        }
        println!(
            "render {:30} ---> {}",
            tpl_path.display(),
            dst_path.display()
        );

        Ok(())
    }
}
fn json_to_gtmpl(value: &serde_json::Value) -> gtmpl::Value {
    match value {
        serde_json::Value::Null => gtmpl::Value::Nil,
        serde_json::Value::Bool(b) => gtmpl::Value::from(*b),
        serde_json::Value::Number(n) => {
            if n.is_f64() {
                gtmpl::Value::from(n.as_f64().unwrap())
            } else if n.is_i64() {
                gtmpl::Value::from(n.as_i64().unwrap())
            } else {
                gtmpl::Value::from(n.as_u64().unwrap())
            }
        }
        serde_json::Value::String(s) => gtmpl::Value::from(s.as_str()),
        serde_json::Value::Array(arr) => {
            gtmpl::Value::from(arr.iter().map(json_to_gtmpl).collect::<Vec<_>>())
        }
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_gtmpl(v));
            }
            gtmpl::Value::from(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::setting::TemplatePath;
    use orion_error::TestAssert;
    use tempfile::tempdir;

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

        // 执行渲染 (注意：当前代码中 Helm 引擎未实现，这里测试路径处理逻辑)
        let _result = TplGtmpl::render_path(
            &helm_dir,
            &out_dir,
            &root_dir.join("value.json"), // 使用 values.yaml 作为数据源
            &setting,
        )
        .unwrap();
    }

    #[test]
    fn test_gtmpl_complex_data() {
        let tmp_dir = tempdir().unwrap();
        let tpl_file = tmp_dir.path().join("template.gtpl");
        let data_file = tmp_dir.path().join("data.json");
        let output_file = tmp_dir.path().join("output.txt");

        std::fs::write(&tpl_file, "User: {{.user.name}}, Age: {{.user.age}}").unwrap();
        std::fs::write(&data_file, r#"{"user": {"name": "Alice", "age": 30}}"#).unwrap();

        let result = TplGtmpl::render_path(
            &tpl_file,
            &output_file,
            &data_file,
            &TemplatePath::default(),
        );

        assert!(result.is_ok());
        assert_eq!(
            std::fs::read_to_string(output_file).unwrap(),
            "User: Alice, Age: 30"
        );
    }
}
