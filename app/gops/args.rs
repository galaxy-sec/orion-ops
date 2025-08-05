use clap::{ArgAction, Args, Parser};
use derive_getters::Getters;
use galaxy_ops::infra::DfxArgsGetter;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gops")]
#[command(
    version,
    about,
    long_about = "Galaxy Operations System - 系统操作管理工具

用于管理系统配置、导入模块、更新引用等操作的核心工具。"
)]
pub enum GInsCmd {
    /// 创建新的系统配置
    ///
    /// 根据提供的参数创建新的系统配置模板
    New(NewArgs),
    /// 导入外部模块到当前系统
    ///
    /// 从指定路径导入模块配置并集成到当前系统
    Import(ImportArgs),
    /// 更新系统模块和引用
    ///
    /// 更新系统模块的引用、依赖关系等配置信息
    Update(UpdateArgs),
    /// 本地化模块配置
    ///
    /// 将模块配置本地化，适配当前环境
    Localize(LocalArgs),
    /// 系统设置管理
    ///
    /// 管理系统级别的配置设置
    Setting(SettingArgs),
}

#[derive(Debug, Args, Getters)]
pub struct SettingArgs {
    /// 调试输出级别
    ///
    /// 设置调试信息的详细程度：
    /// - 0: 无调试输出
    /// - 1: 基础调试信息
    /// - 2: 详细调试信息
    /// - 3: 完整调试信息
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// 日志配置
    ///
    /// 配置日志输出格式和级别，格式：模块=级别,模块=级别
    /// 例如：--log cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
}
impl DfxArgsGetter for SettingArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}

#[derive(Debug, Args, Getters)]
pub struct NewArgs {
    /// 系统配置名称
    ///
    /// 新创建的系统配置的唯一标识名称
    #[arg(short, long, help = "系统配置名称")]
    pub(crate) name: String,
}

#[derive(Debug, Args, Getters)]
pub struct UpdateArgs {
    /// 调试输出级别
    ///
    /// 设置调试信息的详细程度：
    /// - 0: 无调试输出
    /// - 1: 基础调试信息
    /// - 2: 详细调试信息
    /// - 3: 完整调试信息
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// 日志配置
    ///
    /// 配置日志输出格式和级别，格式：模块=级别,模块=级别
    /// 例如：--log cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,

    /// 强制更新级别
    ///
    /// 强制更新远程git仓库：
    /// - 0: 不强制更新
    /// - 1: 强制更新引用
    /// - 2: 强制更新依赖
    /// - 3: 强制更新所有内容
    #[arg(short = 'f', long = "force", default_value = "0")]
    pub force: usize,
}

impl DfxArgsGetter for UpdateArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}

#[derive(Debug, Args, Getters)]
pub struct ImportArgs {
    /// 调试输出级别
    ///
    /// 设置调试信息的详细程度：
    /// - 0: 无调试输出
    /// - 1: 基础调试信息
    /// - 2: 详细调试信息
    /// - 3: 完整调试信息
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// 日志配置
    ///
    /// 配置日志输出格式和级别，格式：模块=级别,模块=级别
    /// 例如：--log cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,

    /// 强制更新级别
    ///
    /// 强制更新远程git仓库：
    /// - 0: 不强制更新
    /// - 1: 强制更新引用
    /// - 2: 强制更新依赖
    /// - 3: 强制更新所有内容
    #[arg(short = 'f', long = "force", default_value = "0")]
    pub force: usize,

    /// 导入路径
    ///
    /// 要导入的模块所在的路径，可以是相对路径或绝对路径
    #[arg(short = 'p', long = "path", help = "模块导入路径")]
    pub path: String,
}

impl DfxArgsGetter for ImportArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}

#[derive(Debug, Args, Getters)]
pub struct LocalArgs {
    /// 调试输出级别
    ///
    /// 设置调试信息的详细程度：
    /// - 0: 无调试输出
    /// - 1: 基础调试信息
    /// - 2: 详细调试信息
    /// - 3: 完整调试信息
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// 日志配置
    ///
    /// 配置日志输出格式和级别，格式：模块=级别,模块=级别
    /// 例如：--log cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,

    /// 值文件路径
    ///
    /// 指定用于本地化的值文件路径，通常为YAML格式
    /// 例如：--value cicd_value.yml
    #[arg(long = "value", help = "本地化值文件路径")]
    pub value: Option<String>,

    /// 使用默认模块配置
    ///
    /// 启用默认模块模式，不使用用户自定义的value.yml文件
    #[arg(long = "default", default_value = "false" , action = ArgAction::SetTrue, help = "使用默认模块配置")]
    pub use_default_value: bool,
}
impl DfxArgsGetter for LocalArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}
