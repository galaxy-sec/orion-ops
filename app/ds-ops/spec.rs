use orion_error::{ErrorConv, ErrorOwe};
use orion_syspec::error::SpecResult;
use orion_syspec::infra::configure_dfx_logging;
use orion_syspec::module::proj::load_project_global_value;
use orion_syspec::ops_prj::proj::OpsProject;
use orion_syspec::types::LocalizeOptions;
use orion_variate::path::make_new_path;
use orion_variate::update::UpdateOptions;
use orion_variate::vars::ValueDict;

use crate::args::GInsCmd;

pub async fn do_ins_cmd(cmd: GInsCmd) -> SpecResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        GInsCmd::New(args) => {
            let new_prj = current_dir.join(args.name());
            make_new_path(&new_prj).owe_res()?;
            let spec = OpsProject::make_new(&new_prj, args.name()).err_conv()?;
            spec.save().err_conv()?;
        }
        GInsCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let options = UpdateOptions::from((dfx.force, ValueDict::default()));
            let spec = OpsProject::load(&current_dir).err_conv()?;
            spec.update(&options).await.err_conv()?;
        }
        GInsCmd::Localize(args) => {
            configure_dfx_logging(&args);
            let spec = OpsProject::load(&current_dir).err_conv()?;
            let dict = load_project_global_value(spec.root_local(), args.value())?;
            spec.localize(LocalizeOptions::new(dict, args.use_default_value))
                .await
                .err_conv()?;
        }
    }
    Ok(())
}
