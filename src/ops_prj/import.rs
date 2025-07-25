use std::path::{Path, PathBuf};

use fs_extra::dir::{move_dir, CopyOptions};
use orion_infra::path::make_clean_path;
use orion_variate::{
    types::LocalUpdate,
    update::UpdateOptions,
    vars::{EnvEvalable, ValueDict},
};

use crate::{
    ops_prj::proj::OpsProject, package::{
        archive::decompress,
        types::{build_pkg, convert_addr, PackageType},
    }, system::spec::SysModelSpec, types::AnyResult
};

impl OpsProject {
    pub async fn import_package(&self,path: &str, up_opt: &UpdateOptions) -> AnyResult<()> {
        // 1. 解析地址
        let addr = convert_addr(path);

        // 2.更新到本地目路
        // 本地路径： ${HOME}/ds-build/
        let work_path = PathBuf::from(
            "${HOME}/ds-package"
                .to_string()
                .env_eval(&ValueDict::default()),
        );

        let up_unit = addr.update_local(&work_path, up_opt).await?;
        let package = build_pkg(path);
        let sys_src = match package {
            //tar.gz ,tgz
            PackageType::Bin(bin_package) => {
                let out_path = work_path.join(bin_package.name());
                make_clean_path(&out_path)?;
                decompress(up_unit.position(), out_path.clone())?;
                out_path
            }
            PackageType::Git(git_package) =>  {
                up_unit.position().to_path_buf()
            }
        };
        let sys_spec = SysModelSpec::load_from(&sys_src.join("sys")) ?;

        // 3.获得sys pakage

        // 4. 导入到 工作目录
        let sys_dst = self.root_local().join( sys_spec.define().name());
        move_dir(sys_src, sys_dst, &CopyOptions::new())?;
        // 5. 提供系统包的信息， 包组所有组件。

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use orion_error::TestAssert;
    use orion_variate::{tools::test_init, update::UpdateOptions};

    use crate::const_vars::EXAMPLE_ROOT;

    use super::*;

    #[tokio::test]
    async fn import_pkg() {
        test_init();
        let prj_path = PathBuf::from(EXAMPLE_ROOT).join("dev-mac-env");
         let mut project = OpsProject::load(&prj_path).assert();
        let path = "${HOME}/ds-build/mac-devkit-0.1.6.tar.gz".to_string().env_eval(&ValueDict::default());
         project.import_package(path.as_str(), &UpdateOptions::for_test()).await.assert();

        //import_package(path,&UpdateOptions::for_test()).await.assert();
        //sys_setting();
    }
}
