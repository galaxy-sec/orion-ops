use clap::{ArgAction, Parser};
use derive_getters::Getters;
use galaxy_ops::infra::DfxArgsGetter;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gsys")]
#[command(version, about = "Galaxy System Management Tool", long_about = "A comprehensive tool for managing Galaxy system configurations, including creating new system specs, updating existing configurations, and localizing settings for different environments.")]
pub enum GSysCmd {
    /// Create new system operator
    #[command(about = "Create new system operator ", long_about = "Create a new system specification with the given name. This will initialize a new system directory structure with all necessary configuration files and templates.")]
    New(NewArgs),
    /// Update existing system configuration
    #[command(about = "Update system configuration", long_about = "Update an existing system's configuration, specifications, or dependencies. Supports force updates to override existing configurations without confirmation.")]
    Update(UpdateArgs),
    /// Localize system configuration for environment
    #[command(about = "Localize system configuration", long_about = "Generate localized configuration files for the system based on environment-specific values. Useful for adapting system configurations to different deployment environments.")]
    Localize(LocalArgs),
}

#[derive(Debug, Args, Getters)]
pub struct NewArgs {
    /// Name of the new system to create
    #[arg(short, long, help = "System name (alphanumeric with hyphens/underscores)")]
    pub(crate) name: String,
}

#[derive(Debug, Args, Getters)]
pub struct UpdateArgs {
    /// Enable debug output with specified level (0-4)
    #[arg(short = 'd', long = "debug", default_value = "0", help = "Debug level: 0=off, 1=basic, 2=verbose, 3=trace, 4=full")]
    pub debug: usize,
    /// Configure logging output format and levels
    #[arg(long = "log", help = "Configure logging: eg --log cmd=debug,parse=info")]
    pub log: Option<String>,

    /// Force update level (0-3)
    #[arg(short = 'f', long = "force", default_value = "0", help = "Force update: 0=normal, 1=skip confirmation, 2=overwrite files, 3=force git pull")]
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
    /// Enable debug output with specified level (0-4)
    #[arg(short = 'd', long = "debug", default_value = "0", help = "Debug level: 0=off, 1=basic, 2=verbose, 3=trace, 4=full")]
    pub debug: usize,
    /// Configure logging output format and levels
    #[arg(long = "log", help = "Configure logging: eg --log cmd=debug,parse=info")]
    pub log: Option<String>,

    /// Path to values file for localization
    #[arg(long = "value", help = "Path to YAML/JSON file containing environment-specific values")]
    pub value: Option<String>,

    /// Use default values instead of user-provided value.yml
    #[arg(long = "default", default_value = "false" , action = ArgAction::SetTrue, help = "Use built-in default values instead of user-provided value.yml")]
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
