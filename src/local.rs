use crate::{
    error::MainError,
    module::{
        localize::LocalizeTemplate,
        setting::{Setting, TemplateConfig},
    },
    predule::*,
    types::{Accessor, Localizable, LocalizeOptions, RefUpdateable, ValuePath},
};
use async_trait::async_trait;
use derive_more::Deref;
use orion_error::UvsResFrom;
use orion_infra::auto_exit_log;
use orion_variate::update::DownloadOptions;

#[derive(Getters, Clone, Debug, Serialize, Deserialize)]
pub struct LocalizePath {
    src: PathBuf,
    dst: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    setting: Option<Setting>,
}
impl LocalizePath {
    pub fn example() -> Self {
        Self {
            src: PathBuf::from("${GXL_PRJ_ROOT}/sys/setting/test.md"),
            dst: PathBuf::from("${GXL_RPJ_ROOT}/sys/mods/test.md"),
            setting: Some(Setting::example()),
        }
    }
    pub fn of_module(module: &str, model: &str) -> Self {
        Self {
            src: PathBuf::from(format!("${{GXL_PRJ_ROOT}}/sys/setting/{module}")),
            dst: PathBuf::from(format!(
                "${{GXL_PRJ_ROOT}}/sys/mods/{module}/{model}/local/",
            )),
            setting: None,
        }
    }
}

#[derive(Getters, Clone, Debug, Default, Serialize, Deserialize, Deref)]
#[serde(transparent)]
pub struct LocalizeSet {
    items: Vec<LocalizePath>,
}

impl LocalizeSet {
    pub fn example() -> Self {
        Self {
            items: vec![
                LocalizePath {
                    src: PathBuf::from("/opt/galaxy/templates/nginx.conf"),
                    dst: PathBuf::from("/etc/nginx/nginx.conf"),
                    setting: Some(Setting::example()),
                },
                LocalizePath {
                    src: PathBuf::from("/opt/galaxy/static/logo.png"),
                    dst: PathBuf::from("/var/www/html/assets/logo.png"),
                    setting: None,
                },
            ],
        }
    }
}

#[async_trait]
impl RefUpdateable<()> for LocalizeSet {
    async fn update_local(
        &self,
        _accessor: Accessor,
        _path: &Path,
        _options: &DownloadOptions,
    ) -> MainResult<()> {
        // For now, template paths are handled as local files
        Ok(())
    }
}

#[async_trait]
impl Localizable for LocalizeSet {
    async fn localize(
        &self,
        dst_path: Option<ValuePath>,
        options: LocalizeOptions,
    ) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(target: "sys-localize", "Localizing {} paths for sys_local", self.items.len()),
            error!(target: "sys-localize", "Failed to localize sys_local paths")
        );

        for item in &self.items {
            item.localize(dst_path.clone(), options.clone()).await?;
        }

        flag.mark_suc();
        Ok(())
    }
}

#[async_trait]
impl Localizable for LocalizePath {
    async fn localize(
        &self,
        val_path: Option<ValuePath>,
        _options: LocalizeOptions,
    ) -> MainResult<()> {
        let mut flag = auto_exit_log!(
            info!(target: "sys-localize", "sys-path localize {} success!", self.dst.display()),
            error!(target: "sys-localize", "sys-path localize {} fail!", self.dst.display())
        );
        if !self.src().exists() {
            info!(target: "sys-localize", "path localize ignore!\n src not exists : {}", self.dst.display());
            flag.mark_suc();
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = self.dst.parent() {
            std::fs::create_dir_all(parent).owe_res()?;
        }
        let mut ctx = WithContext::want("sys-path localize");
        ctx.with_path("dst", &self.dst);
        ctx.with_path("src", &self.src);

        // Handle template configuration if available
        if let (Some(setting), Some(value_file)) =
            (self.setting.clone().or(Some(Setting::default())), val_path)
        {
            if !value_file.path().exists() {
                return MainError::from_res(format!(
                    "sys value file not exists: {}",
                    value_file.path().display()
                ))
                .err();
            }
            let tpl_path_opt = setting
                .localize()
                .clone()
                .and_then(|x| x.templatize_path().clone())
                .map(|x| x.export_paths(self.dst()));

            let tpl_path = tpl_path_opt.unwrap_or_default();
            let tpl_custom = setting
                .localize()
                .clone()
                .and_then(|x| x.templatize_cust().clone())
                .map(TemplateConfig::from);

            let localizer = if let Some(cust) = tpl_custom {
                LocalizeTemplate::new(cust)
            } else {
                LocalizeTemplate::default()
            };
            localizer
                .render_path(self.src(), &self.dst, value_file.path(), &tpl_path)
                .with(&ctx)?;
        } else {
            return MainError::from_res("sys value file miss".into()).err();
        }

        flag.mark_suc();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orion_common::serde::JsonAble;
    use orion_error::TestAssert;
    use orion_variate::vars::ValueDict;
    use std::{fs, io::Write};
    use tempfile::{NamedTempFile, tempdir};

    fn create_test_localize_path() -> (LocalizePath, NamedTempFile, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let source_file = NamedTempFile::new().unwrap();
        let dest_path = temp_dir.path().join("dest.txt");

        writeln!(source_file.as_file(), "test content").unwrap();

        let localize_path = LocalizePath {
            src: source_file.path().to_path_buf(),
            dst: dest_path.clone(),
            setting: None,
        };

        (localize_path, source_file, temp_dir)
    }

    #[tokio::test]
    async fn test_localize_path_basic() {
        let (localize_path, _source_file, temp_dir) = create_test_localize_path();

        let value_path = temp_dir.path().join("used.json");
        ValueDict::default().save_json(&value_path).assert();
        // Test basic file localization
        let result = localize_path
            .localize(
                Some(ValuePath::new(&value_path)),
                LocalizeOptions::default(),
            )
            .await;
        assert!(result.is_ok());
        assert!(localize_path.dst.exists());

        // Verify file content
        let content = fs::read_to_string(&localize_path.dst).unwrap();
        assert_eq!(content.trim(), "test content");
    }

    #[tokio::test]
    async fn test_localize_set_multiple_files() {
        let temp_dir = tempdir().unwrap();

        // Create source files
        let file1 = temp_dir.path().join("source1.txt");
        let file2 = temp_dir.path().join("source2.txt");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let localize_set = LocalizeSet {
            items: vec![
                LocalizePath {
                    src: file1.clone(),
                    dst: temp_dir.path().join("dest1.txt"),
                    setting: None,
                },
                LocalizePath {
                    src: file2.clone(),
                    dst: temp_dir.path().join("dest2.txt"),
                    setting: None,
                },
            ],
        };
        let value_path = temp_dir.path().join("used.json");
        ValueDict::default().save_json(&value_path).assert();
        let result = localize_set
            .localize(
                Some(ValuePath::new(&value_path)),
                LocalizeOptions::default(),
            )
            .await;
        assert!(result.is_ok());

        // Verify both files were localized
        assert!(temp_dir.path().join("dest1.txt").exists());
        assert!(temp_dir.path().join("dest2.txt").exists());

        assert_eq!(
            fs::read_to_string(temp_dir.path().join("dest1.txt"))
                .unwrap()
                .trim(),
            "content1"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("dest2.txt"))
                .unwrap()
                .trim(),
            "content2"
        );
    }

    #[test]
    fn test_localize_path_struct() {
        let path1 = PathBuf::from("/src/file.txt");
        let path2 = PathBuf::from("/dst/file.txt");

        let localize_path = LocalizePath {
            src: path1.clone(),
            dst: path2.clone(),
            setting: None,
        };

        assert_eq!(localize_path.src(), &path1);
        assert_eq!(localize_path.dst(), &path2);
        assert!(localize_path.setting().is_none());
    }

    #[test]
    fn test_localize_set_struct() {
        let path1 = LocalizePath {
            src: PathBuf::from("/src1.txt"),
            dst: PathBuf::from("/dst1.txt"),
            setting: None,
        };
        let path2 = LocalizePath {
            src: PathBuf::from("/src2.txt"),
            dst: PathBuf::from("/dst2.txt"),
            setting: None,
        };

        let set = LocalizeSet {
            items: vec![path1, path2],
        };

        assert_eq!(set.items.len(), 2);
        assert_eq!(set.len(), 2);
    }
}
