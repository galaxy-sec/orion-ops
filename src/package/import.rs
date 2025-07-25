use std::path::{Path, PathBuf};

use orion_variate::{types::LocalUpdate, update::UpdateOptions, vars::{EnvEvalable, ValueDict}};

use crate::{package::types::convert_addr, types::AnyResult};

pub async  fn import_package(path: &str, up_opt: &UpdateOptions) -> AnyResult<()> {
    

    // 1. 解析地址
    let addr = convert_addr(path);

    // 2.更新到本地目路
    // 本地路径： ${HOME}/ds-build/
    let work_path = PathBuf::from("${HOME}/ds-build".to_string().env_eval(&ValueDict::default()));

    addr.update_local(&work_path,up_opt).await? ;
    // 3.获得sys pakage  

    // 4. 导入到 工作目录

    // 5. 提供系统包的信息， 包组所有组件。

    Ok(())
}

#[cfg(test)]
mod test {
    use orion_error::TestAssert;
    use orion_variate::update::UpdateOptions;

    use super::import_package;

    #[tokio::test]
    async fn import_pkg() {
        let path = "${HOME}/ds-build/mac-devkit-0.1.5.tar.gz";
        import_package(path,&UpdateOptions::for_test()).await.assert();
        //sys_setting();
    }
}
