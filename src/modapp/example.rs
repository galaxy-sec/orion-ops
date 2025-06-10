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
    mod_list.add_ref(
        ModuleSpecRef::from(
            mod_name,
            GitAddr::from("https://e.coding.net/dy-sec/galaxy-open/modspec").path("nginx"),
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
