use std::path::PathBuf;

use orion_error::ErrorConv;
use orion_syspec::error::SpecResult;
use orion_syspec::infra::configure_dfx_logging;
use orion_syspec::system::proj::SysProject;
use orion_syspec::tools::make_new_path;
use orion_syspec::types::{Localizable, UpdateOptions};

use crate::args::GSysCmd;

pub async fn do_sys_cmd(cmd: GSysCmd) -> SpecResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        GSysCmd::New(args) => {
            let new_prj = current_dir.join(args.name());
            make_new_path(&new_prj)?;
            let spec = SysProject::make_new(&new_prj, args.name(), "https://").err_conv()?;
            spec.save().err_conv()?;
        }
        GSysCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = SysProject::load(&current_dir).err_conv()?;
            spec.update(&UpdateOptions::default()).await.err_conv()?;
        }
        GSysCmd::Localize(args) => {
            configure_dfx_logging(&args);
            let spec = SysProject::load(&current_dir).err_conv()?;
            spec.localize(None, args.value.map(PathBuf::from))
                .await
                .err_conv()?;
        }
    }
    Ok(())
}
