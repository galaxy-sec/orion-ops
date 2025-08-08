use std::path::{Path, PathBuf};

use galaxy_ops::{
    accessor::accessor_for_test,
    const_vars::{SYS_MODEL_PRJ_ROOT, WORKINS_PRJ_ROOT},
    error::MainResult,
    module::depend::{Dependency, DependencySet},
    ops_prj::proj::OpsProject,
    system::{proj::SysProject, spec::SysModelSpec},
    types::{InsUpdateable, LocalizeOptions, RefUpdateable},
};
use orion_error::{ErrorOwe, TestAssertWithMsg};
use orion_infra::path::make_clean_path;
use orion_variate::{
    addr::{Address, HttpResource, types::PathTemplate},
    archive::compress,
    tools::test_init,
    update::DownloadOptions,
};
#[tokio::test]
async fn test_full_flow() -> MainResult<()> {
    test_init();
    let sys_proj = make_sys_prj_example().await?;
    let out_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("example_sys_x.tar.gz");
    compress(sys_proj.root_local(), &out_path).owe_sys()?;
    let mut ops_proj = make_workins_example().await?;
    let accessor = accessor_for_test();
    ops_proj
        .import_sys(
            accessor.clone(),
            out_path.display().to_string().as_str(),
            &DownloadOptions::for_test(),
        )
        .await?;
    let sys_path = ops_proj.root_local().join("example_sys_x");
    let sys_proj = SysProject::load(&sys_path)?;
    sys_proj
        .update_local(accessor, &sys_path, &DownloadOptions::default())
        .await?;
    sys_proj.localize(LocalizeOptions::for_test()).await?;
    Ok(())
    //sys_proj.
}
async fn make_workins_example() -> MainResult<OpsProject> {
    test_init();
    let prj_path = PathBuf::from(WORKINS_PRJ_ROOT).join("workins_sys_x");
    make_clean_path(&prj_path).owe_logic()?;
    let project = OpsProject::for_test("workins_sys_x").assert("make workins");
    project.save().assert("save workins_prj");
    let project = OpsProject::load(&prj_path).assert("workins-prj");
    let accessor = accessor_for_test();
    let project = project
        .update_local(accessor, &prj_path, &DownloadOptions::default())
        .await
        .assert("spec.update_local");
    Ok(project)
}

async fn make_sys_prj_example() -> MainResult<SysProject> {
    let prj_path = PathBuf::from(SYS_MODEL_PRJ_ROOT).join("example_sys_x");
    make_clean_path(&prj_path).owe_logic()?;
    let project = make_sys_prj_testins(&prj_path).assert("make cust");
    if prj_path.exists() {
        std::fs::remove_dir_all(&prj_path).assert("ok");
    }
    std::fs::create_dir_all(&prj_path).assert("yes");
    project.save().assert("save dss_prj");
    let project = SysProject::load(&prj_path).assert("dss-project");
    let accessor = accessor_for_test();
    project
        .update_local(accessor, &prj_path, &DownloadOptions::default())
        .await
        .assert("spec.update_local");
    project
        .localize(LocalizeOptions::for_test())
        .await
        .assert("spec.localize");
    Ok(project)
}

fn make_sys_prj_testins(prj_path: &Path) -> MainResult<SysProject> {
    let mod_spec = SysModelSpec::for_example("example_sys_x")?;
    let mut res = DependencySet::default();
    res.push(
        Dependency::new(
            Address::from(HttpResource::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            PathTemplate::from(prj_path.join("test_res")),
        )
        .with_rename("bit-common"),
    );
    Ok(SysProject::new(mod_spec, res, prj_path.to_path_buf()))
}
