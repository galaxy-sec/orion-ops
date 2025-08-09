use std::path::PathBuf;

use fs_extra::dir::{CopyOptions, move_dir};
use orion_common::serde::Configable;
use orion_error::{ErrorOwe, ErrorWith, UvsConfFrom};
use orion_infra::path::make_clean_path;
use orion_variate::{
    archive::decompress,
    types::ResourceDownloader,
    update::DownloadOptions,
    vars::{EnvEvalable, ValueDict, VarCollection},
};

use crate::{
    artifact::types::{PackageType, build_pkg, convert_addr},
    error::{MainError, MainResult},
    ops_prj::{proj::OpsProject, system::OpsSystem},
    system::spec::SysModelSpec,
    types::Accessor,
};

impl OpsProject {
    pub async fn import_sys(
        &mut self,
        accessor: Accessor,
        path: &str,
        up_opt: &DownloadOptions,
    ) -> MainResult<SysModelSpec> {
        // 1. 解析地址
        let addr = convert_addr(path);

        // 2.更新到本地目路
        // 本地路径： ${HOME}/ds-build/
        let work_path = PathBuf::from(
            "${HOME}/ds-package"
                .to_string()
                .env_eval(&ValueDict::default()),
        );

        let up_unit = accessor
            .download_to_local(&addr, &work_path, up_opt)
            .await
            .owe_data()?;
        let package = build_pkg(path);
        let sys_src = match package {
            //tar.gz ,tgz
            PackageType::Bin(bin_package) => {
                let out_path = work_path.join(bin_package.name());
                make_clean_path(&out_path).owe_res()?;
                decompress(up_unit.position(), out_path.clone())
                    .owe_sys()
                    .want("decompress tar.gz")
                    .with(up_unit.position().display().to_string())?;
                out_path
            }
            PackageType::Git(_git_package) => up_unit.position().to_path_buf(),
        };
        let sys_spec = SysModelSpec::load_from(&sys_src.join("sys"))?;

        let ops_sys = OpsSystem::new(sys_spec.define().clone(), addr);
        self.import_ops_sys(ops_sys);
        // 3.获得sys pakage

        // 4. 导入到 工作目录
        let sys_dst_root = self.root_local();
        //if let Some(last_name) = sys_src.iter().last() {
        if let Some(last_name) = sys_src.iter().next_back() {
            let sys_dst_path = sys_dst_root.join(last_name);
            let sys_new_path = sys_dst_root.join(sys_spec.define().name());
            if sys_dst_path.exists() {
                std::fs::remove_dir_all(&sys_dst_path).owe_res()?;
            }
            if sys_new_path.exists() {
                std::fs::remove_dir_all(&sys_new_path).owe_res()?;
            }
            move_dir(sys_src, sys_dst_root, &CopyOptions::new()).owe_res()?;
            std::fs::rename(sys_dst_path, sys_new_path).owe_res()?;
            let value_path = self
                .root_local()
                .join("values")
                .join(sys_spec.define().name());
            let value_link = self
                .root_local()
                .join(sys_spec.define().name())
                .join("values");
            let value_file = value_path.join("value.yml");
            if !value_file.exists() {
                std::fs::create_dir(&value_path).owe_res()?;
                ValueDict::default().save_conf(&value_file).owe_res()?;
            }
            if !value_link.exists() {
                std::os::unix::fs::symlink(&value_path, &value_link)
                    .owe_res()
                    .with(&value_link)?;
            }
        } else {
            MainError::from_conf(format!(
                "import package failed, bad path: {}",
                sys_src.display()
            ));
        }
        self.save()?;
        // 5. 提供系统包的信息， 包组所有组件。
        Ok(sys_spec)
    }
    pub fn ia_setting(&self) -> MainResult<()> {
        use inquire::{Confirm, Text};

        for i in self.ops_target().iter() {
            let vars_path = self.root_local().join(i.sys().name()).join("sys/vars.yml");
            let value_path = self
                .root_local()
                .join("values")
                .join(i.sys().name())
                .join("value.yml");
            let vars_vec = VarCollection::from_conf(&vars_path).owe_res()?;
            let mut vals_dict = ValueDict::from_conf(&value_path).owe_res()?;

            // 通过交互模式设定vars的值
            println!("Setting variables for {}", i.sys().name());

            for var in vars_vec.vars() {
                let prompt = if let Some(desp) = var.desp() {
                    format!("{}\n{desp}", var.name())
                } else {
                    var.name().to_string()
                };
                let mut default_value = var.value();
                let value_str = Text::new(&prompt)
                    .with_default(&var.value().to_string())
                    .prompt()
                    .owe_data()?;
                default_value.update_by_str(value_str.as_str()).owe_data()?;
                vals_dict.insert(var.name().to_string(), default_value);
            }

            // 如果用户确认保存更改
            if Confirm::new("Do you want to save these changes?")
                .prompt()
                .owe_data()?
            {
                // 保存修改后的vars到文件
                // vars.save_to_file(&vars_path)?; // 假设的方法
                println!("Changes saved to {}", vars_path.display());
                vals_dict.save_conf(&value_path).owe_res()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use orion_error::TestAssert;
    use orion_variate::{tools::test_init, update::DownloadOptions};

    use crate::{accessor::accessor_for_test, const_vars::EXAMPLE_ROOT};

    use super::*;

    #[ignore = "need interactive run"]
    #[tokio::test]
    async fn import_pkg() {
        test_init();
        let prj_path = PathBuf::from(EXAMPLE_ROOT).join("dev-mac-env");
        let mut project = OpsProject::load(&prj_path).assert();
        let path = "${HOME}/ds-build/mac-devkit-0.1.6.tar.gz"
            .to_string()
            .env_eval(&ValueDict::default());
        let accessor = accessor_for_test();
        let sys_spec = project
            .import_sys(accessor, path.as_str(), &DownloadOptions::for_test())
            .await
            .assert();
        println!("{}", serde_json::to_string(&sys_spec).assert());
    }
    #[ignore = "need interactive run"]
    #[tokio::test]
    async fn ia_setting() {
        test_init();
        let prj_path = PathBuf::from(EXAMPLE_ROOT).join("dev-mac-env");
        let project = OpsProject::load(&prj_path).assert();
        project.ia_setting().assert();
    }
}
