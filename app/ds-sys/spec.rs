use inquire::Select;
use orion_error::{ErrorConv, ErrorOwe};
use orion_infra::path::make_new_path;
use orion_ops::error::MainResult;
use orion_ops::infra::configure_dfx_logging;
use orion_ops::module::ModelSTD;
use orion_ops::module::proj::load_project_global_value;
use orion_ops::system::proj::SysProject;
use orion_ops::types::LocalizeOptions;
use orion_variate::update::UpdateOptions;
use orion_variate::vars::ValueDict;

use crate::args::GSysCmd;

fn ia_model_std() -> MainResult<ModelSTD> {
    let support_models = ModelSTD::support();

    // 准备选项列表
    let options: Vec<String> = support_models
        .iter()
        .map(|model| format!("{model}"))
        .collect();

    // 添加使用当前系统的选项
    let all_options = options;

    let selection = Select::new("请选择系统型号配置:", all_options.clone())
        .prompt()
        .unwrap();

    // 从预定义选项中选择
    let index = all_options.iter().position(|s| s == &selection).unwrap();
    if index < support_models.len() {
        Ok(support_models[index].clone())
    } else {
        Ok(ModelSTD::from_cur_sys()) // 兜底处理
    }
}

pub async fn do_sys_cmd(cmd: GSysCmd) -> MainResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        GSysCmd::New(args) => {
            let new_prj = current_dir.join(args.name());
            make_new_path(&new_prj).owe_res()?;
            let model_in = ia_model_std()?;
            let spec = SysProject::make_new(&new_prj, args.name(), model_in).err_conv()?;
            spec.save().err_conv()?;
        }
        GSysCmd::Update(dfx) => {
            configure_dfx_logging(&dfx);
            let options = UpdateOptions::from((dfx.force, ValueDict::default()));
            let spec = SysProject::load(&current_dir).err_conv()?;
            spec.update(&options).await.err_conv()?;
        }
        GSysCmd::Localize(args) => {
            configure_dfx_logging(&args);
            let spec = SysProject::load(&current_dir).err_conv()?;
            let dict = load_project_global_value(spec.root_local(), args.value())?;
            spec.localize(LocalizeOptions::new(dict, args.use_default_value))
                .await
                .err_conv()?;
        }
    }
    Ok(())
}
