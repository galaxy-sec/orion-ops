use std::path::{Path, PathBuf};

use orion_error::{ErrorOwe, TestAssertWithMsg};
use orion_infra::path::make_clean_path;
use orion_ops::{
    const_vars::SYS_MODEL_PRJ_ROOT,
    error::MainResult,
    module::depend::{Dependency, DependencySet},
    system::{proj::SysProject, spec::SysModelSpec},
    types::LocalizeOptions,
};
use orion_variate::{
    addr::{AddrType, GitAddr, types::EnvVarPath},
    archive::compress,
    tools::test_init,
    update::UpdateOptions,
};
#[tokio::test]
async fn test_all() -> MainResult<()> {
    let sys_proj = make_sys_prj_example().await?;
    let out_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("example_sys_x.tar.gz");
    compress(sys_proj.root_local(), &out_path).owe_sys()?;
    Ok(())
    //sys_proj.
}
async fn make_sys_prj_example() -> MainResult<SysProject> {
    test_init();

    let prj_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("example_sys_x");
    make_clean_path(&prj_path).owe_logic()?;
    let project = make_sys_prj_testins(&prj_path).assert("make cust");
    if prj_path.exists() {
        std::fs::remove_dir_all(&prj_path).assert("ok");
    }
    std::fs::create_dir_all(&prj_path).assert("yes");
    project.save().assert("save dss_prj");
    let project = SysProject::load(&prj_path).assert("dss-project");
    project
        .update(&UpdateOptions::default())
        .await
        .assert("spec.update_local");
    project
        .localize(LocalizeOptions::for_test())
        .await
        .assert("spec.localize");
    Ok(project)
}

fn make_sys_prj_testins(prj_path: &Path) -> MainResult<SysProject> {
    let mod_spec = SysModelSpec::for_example("exmaple_sys_x")?;
    let mut res = DependencySet::default();
    res.push(
        Dependency::new(
            AddrType::from(GitAddr::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            EnvVarPath::from(prj_path.join("test_res")),
        )
        .with_rename("bit-common"),
    );
    Ok(SysProject::new(mod_spec, res, prj_path.to_path_buf()))
}
