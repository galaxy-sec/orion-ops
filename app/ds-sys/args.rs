use clap::Parser;
use derive_getters::Getters;
use orion_syspec::{infra::DfxArgsGetter, update::DurationLevel};

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ds-sys")]
#[command(version, about)]
pub enum GSysCmd {
    New(NewArgs),
    Update(UpdateArgs),
    Localize(LocalArgs),
}

#[derive(Debug, Args, Getters)]
pub struct NewArgs {
    #[arg(short, long)]
    pub(crate) name: String,
}

#[derive(Debug, Args, Getters)]
pub struct UpdateArgs {
    ///output run log ;
    ///level : 1,2,3,4
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,

    /// force update;
    /// eg : -f 2;
    /// 1,  2: force update remote git
    #[arg(short = 'f', long = "force", default_value = "0")]
    pub force: usize,
    /// update scope;
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

    /// use vlaue file; eg: --value cicd_value.yml
    #[arg(long = "value")]
    pub value: Option<String>,
}
impl DfxArgsGetter for LocalArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}

#[derive(ValueEnum, Debug, Clone, Default, PartialEq)]
pub enum UpLevelArg {
    ///all
    #[default]
    All,
    ///mod
    Mod,
    ///mod/element
    Elm,
}

impl From<UpLevelArg> for DurationLevel {
    fn from(value: UpLevelArg) -> Self {
        match value {
            UpLevelArg::All => Self::DurLong,
            UpLevelArg::Mod => Self::DurProj,
            UpLevelArg::Elm => Self::DurMod,
        }
    }
}
