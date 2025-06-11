use std::path::PathBuf;

use crate::{
    addr::{AddrType, GitAddr, LocalAddr},
    error::SpecResult,
    module::{
        CpuArch, OsCPE, RunSPC, TargetNode,
        refs::{DependItem, ModuleSpecRef},
    },
    system::ModulesList,
};

use super::modapp::{LocalRes, ModAppProject};

pub fn make_mod_cust_example(prj_path: &PathBuf) -> SpecResult<ModAppProject> {
    let mod_name = "postgresql";
    let mut mod_list = ModulesList::default();
    mod_list.add_ref(
        ModuleSpecRef::from(
            mod_name,
            LocalAddr::from("./mod-spec/postgresql"),
            TargetNode::new(CpuArch::Arm, OsCPE::MAC14, RunSPC::Host),
        )
        .with_enable(false),
    );
    let nginx_mod = "nginx";
    mod_list.add_ref(
        ModuleSpecRef::from(
            nginx_mod,
            GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/modspec").path(nginx_mod),
            TargetNode::new(CpuArch::X86, OsCPE::UBT22, RunSPC::K8S),
        )
        .with_enable(true),
    );

    let mut res = LocalRes::default();
    res.push(
        DependItem::new(
            AddrType::from(GitAddr::from(
                "https://e.coding.net/dy-sec/galaxy-open/bitnami-common.git",
            )),
            prj_path.join("env_res"),
        )
        .with_rename("bit-common"),
    );
    Ok(ModAppProject::new(mod_list, res, prj_path.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use orion_error::TestAssertWithMsg;
    use tempfile::tempdir;

    #[test]
    fn test_make_mod_cust_example() {
        // 1. 准备测试目录
        let prj_dir = tempdir().unwrap();
        let prj_path = prj_dir.path().to_path_buf();

        // 2. 调用被测函数
        let project = make_mod_cust_example(&prj_path).unwrap();
        project.save(&prj_path).assert("save prj");
        let project = ModAppProject::load(&prj_path).assert("load prj");

        // 3. 验证模块列表
        let mod_list = project.modules();
        assert_eq!(mod_list.len(), 2);

        // 验证第一个模块 (postgresql-local)
        let local_mod = mod_list.find("postgresql").unwrap();
        assert_eq!(local_mod.name(), "postgresql");
        assert!(!local_mod.is_enable());
        assert!(matches!(
            local_mod.addr(),
            AddrType::Local(LocalAddr { .. })
        ));
    }
}
