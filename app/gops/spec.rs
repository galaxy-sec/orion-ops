use galaxy_ops::error::MainResult;
use galaxy_ops::infra::configure_dfx_logging;
use galaxy_ops::ops_prj::proj::OpsProject;
use orion_error::{ErrorConv, ErrorOwe};
use orion_infra::path::make_new_path;
use orion_variate::update::DownloadOptions;
use orion_variate::vars::ValueDict;

use crate::args::GInsCmd;

pub async fn do_ins_cmd(cmd: GInsCmd) -> MainResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        GInsCmd::New(args) => {
            let new_prj = current_dir.join(args.name());
            make_new_path(&new_prj).owe_res()?;
            let spec = OpsProject::make_new(&new_prj, args.name()).err_conv()?;
            spec.save().err_conv()?;
        }
        GInsCmd::Import(args) => {
            configure_dfx_logging(&args);
            let options = DownloadOptions::from((args.force, ValueDict::default()));
            let mut prj = OpsProject::load(&current_dir).err_conv()?;
            prj.import_sys(args.path(), &options).await.err_conv()?;
        }
        GInsCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let options = DownloadOptions::from((dfx.force, ValueDict::default()));
            let spec = OpsProject::load(&current_dir).err_conv()?;
            spec.update(&options).await.err_conv()?;
        }
        GInsCmd::Localize(_args) => {
            todo!();
            /*
            configure_dfx_logging(&args);
            let spec = OpsProject::load(&current_dir).err_conv()?;
            let dict = load_project_global_value(spec.root_local(), args.value())?;
            spec.localize(LocalizeOptions::new(dict, args.use_default_value))
                .await
                .err_conv()?;
            */
        }
        GInsCmd::Setting(args) => {
            configure_dfx_logging(&args);
            let spec = OpsProject::load(&current_dir).err_conv()?;
            spec.ia_setting()?;
        }
    }
    Ok(())
}
