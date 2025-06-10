use orion_error::ErrorConv;
use orion_syspec::error::SpecResult;
use orion_syspec::infra::configure_dfx_logging;
use orion_syspec::modapp::example::make_mod_cust_example;
use orion_syspec::modapp::modapp::{ModAppProject, make_mod_cust_testins};
use orion_syspec::module::spec::{make_mod_spec_example, make_mod_spec_new};
use orion_syspec::types::{Localizable, Persistable};
use std::path::PathBuf;

use crate::args::{self};

pub async fn do_mod_cmd(cmd: args::GxModCmd) -> SpecResult<()> {
    let _current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::GxModCmd::Def(subcmd) => return do_spec_cmd(subcmd).await,
        args::GxModCmd::App(subcmd) => return do_cust_cmd(subcmd).await,
    }
}

pub async fn do_spec_cmd(cmd: args::SpecCmd) -> SpecResult<()> {
    let _current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::SpecCmd::Example => {
            let spec = make_mod_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./"), None).err_conv()?;
        }
        args::SpecCmd::New(spec_args) => {
            configure_dfx_logging(&spec_args);
            let spec = make_mod_spec_new(spec_args.name.as_str()).err_conv()?;
            spec.save_to(&PathBuf::from("./"), None).err_conv()?;
        }
    }
    Ok(())
}
pub async fn do_cust_cmd(cmd: args::CustCmd) -> SpecResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::CustCmd::Example => {
            let spec = make_mod_cust_example(&current_dir).err_conv()?;
            spec.save(&current_dir).err_conv()?;
        }
        args::CustCmd::New(spec_args) => {
            configure_dfx_logging(&spec_args);
            todo!();
        }
        args::CustCmd::Localize(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = ModAppProject::load(&current_dir).err_conv()?;
            spec.localize(None).await.err_conv()?;
        }
        args::CustCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = ModAppProject::load(&current_dir).err_conv()?;
            spec.update().await.err_conv()?;
        }
    }
    Ok(())
}
