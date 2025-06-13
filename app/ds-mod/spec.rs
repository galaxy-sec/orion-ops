use orion_error::ErrorConv;
use orion_syspec::app_mod::app::ModAppProject;
use orion_syspec::app_mod::example::make_mod_cust_example;
use orion_syspec::error::SpecResult;
use orion_syspec::infra::configure_dfx_logging;
use orion_syspec::module::spec::{ModuleSpec, make_mod_spec_example, make_mod_spec_new};
use orion_syspec::types::{
    AsyncUpdateable, Localizable, Persistable, RedoLevel, UpdateLevel, UpdateOptions,
};
use std::path::PathBuf;

use crate::args::{self};

pub async fn do_mod_cmd(cmd: args::GxModCmd) -> SpecResult<()> {
    let _current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::GxModCmd::Def(subcmd) => do_spec_cmd(subcmd).await,
        args::GxModCmd::App(subcmd) => do_app_cmd(subcmd).await,
    }
}

pub async fn do_spec_cmd(cmd: args::SpecCmd) -> SpecResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
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
        args::SpecCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = ModuleSpec::load_from(&current_dir).err_conv()?;
            let redo_level = RedoLevel::from(dfx.force);
            spec.update_local(
                &current_dir,
                &UpdateOptions::new(redo_level, UpdateLevel::from(dfx.level)),
            )
            .await
            .err_conv()?;
        }
    }
    Ok(())
}
pub async fn do_app_cmd(cmd: args::AppCmd) -> SpecResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::AppCmd::Example => {
            let spec = make_mod_cust_example(&current_dir).err_conv()?;
            spec.save(&current_dir).err_conv()?;
        }
        args::AppCmd::New(spec_args) => {
            configure_dfx_logging(&spec_args);
            todo!();
        }
        args::AppCmd::Localize(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = ModAppProject::load(&current_dir).err_conv()?;
            spec.localize(None).await.err_conv()?;
        }
        args::AppCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = ModAppProject::load(&current_dir).err_conv()?;
            let redo_level = RedoLevel::from(dfx.force);
            let options = &UpdateOptions::new(redo_level, UpdateLevel::from(dfx.level));
            spec.update(options).await.err_conv()?;
        }
    }
    Ok(())
}
