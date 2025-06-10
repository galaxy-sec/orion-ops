use clap::Parser;
use derive_getters::Getters;
use orion_syspec::infra::DfxArgsGetter;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gmod")]
#[command(version, about)]
pub enum GxModCmd {
    #[command(subcommand)]
    Def(SpecCmd),
    #[command(subcommand)]
    App(CustCmd),
}

#[derive(Debug, Subcommand)] // requires `derive` feature
pub enum SpecCmd {
    Example,
    New(SpecArgs),
}

#[derive(Debug, Subcommand)] // requires `derive` feature
pub enum CustCmd {
    Example,
    New(SpecArgs),
    Update(DfxArgs),
    Localize(DfxArgs),
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
pub struct DfxArgs {
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
}
impl DfxArgsGetter for DfxArgs {
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
