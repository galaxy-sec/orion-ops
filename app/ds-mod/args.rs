use clap::Parser;
use derive_getters::Getters;
use orion_syspec::infra::DfxArgsGetter;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ds-mod")]
#[command(version, about)]
pub enum GxModCmd {
    Example,
    /// define new module spec
    /// eg: ds-mod def new --name mod_name
    New(SpecArgs),
    /// module spec update ref,depends
    Update(UpdateArgs),
    ///localize modules spec
    Localize(LocalArgs),
}

#[derive(Debug, Args, Getters)]
pub struct SpecArgs {
    #[arg(short, long)]
    pub(crate) name: String,

    ///output run log
    ///level : 1,2,3,4
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
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

impl DfxArgsGetter for SpecArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}
