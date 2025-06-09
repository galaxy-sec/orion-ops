use orion_error::ErrorConv;
use orion_syspec::addr::GitAddr;
use orion_syspec::cust::sysproj::{SysCustProject, make_sys_cust_example};
use orion_syspec::error::SpecResult;
use orion_syspec::infra::configure_dfx_logging;
use orion_syspec::system::refs::SysModelSpecRef;
use orion_syspec::types::{AsyncUpdateable, Configable, Localizable, UpdateOptions};

use crate::args::GCustCmd;

pub async fn do_cust_cmd(cmd: GCustCmd) -> SpecResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    let conf_file = current_dir.join("sys_cust-prj.yaml");
    match cmd {
        GCustCmd::Example => {
            let spec = make_sys_cust_example(current_dir).err_conv()?;
            spec.save_conf(&conf_file).err_conv()?;
        }
        GCustCmd::Load(load_args) => {
            configure_dfx_logging(&load_args);
            let target = load_args.path();
            let spec_ref =
                SysModelSpecRef::from(target, GitAddr::from(load_args.repo()).path(target));
            spec_ref
                .update_local(&current_dir, &UpdateOptions::default())
                .await
                .err_conv()?;
        }
        GCustCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = SysCustProject::from_conf(&conf_file).err_conv()?;
            spec.update().await.err_conv()?;
        }
        GCustCmd::Localize(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = SysCustProject::from_conf(&conf_file).err_conv()?;
            spec.localize(None).await.err_conv()?;
        }
    }
    Ok(())
}
