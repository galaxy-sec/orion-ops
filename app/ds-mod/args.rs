use clap::Parser;
use derive_getters::Getters;
use orion_syspec::{infra::DfxArgsGetter, types::UpdateLevel};

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gmod")]
#[command(version, about)]
pub enum GxModCmd {
    #[command(subcommand)]
    Def(SpecCmd),
    #[command(subcommand)]
    App(AppCmd),
}

#[derive(Debug, Subcommand)] // requires `derive` feature
pub enum SpecCmd {
    Example,
    New(SpecArgs),
    Update(UpdateArgs),
}

#[derive(Debug, Subcommand)] // requires `derive` feature
pub enum AppCmd {
    Example,
    New(SpecArgs),
    Update(UpdateArgs),
    Localize(LocalArgs),
}

#[derive(Debug, Args, Getters)]
pub struct SpecArgs {
    #[arg(short, long)]
    pub(crate) name: String,
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
}
#[derive(Debug, Args, Getters)]
pub struct UpdateArgs {
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
    #[clap(value_enum, default_value_t)]
    pub level: UpLevelArg,
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
pub struct LocalArgs {
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
}
impl DfxArgsGetter for LocalArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}

impl DfxArgsGetter for SpecArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}
#[derive(ValueEnum, Debug, Clone, Default, PartialEq)]
pub enum UpLevelArg {
    #[default]
    All,
    Mod,
    Elm,
}

impl From<UpLevelArg> for UpdateLevel {
    fn from(value: UpLevelArg) -> Self {
        match value {
            UpLevelArg::All => Self::All,
            UpLevelArg::Mod => Self::Mod,
            UpLevelArg::Elm => Self::Elm,
        }
    }
}
