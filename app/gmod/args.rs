use clap::{ArgAction, Parser};
use derive_getters::Getters;
use galaxy_ops::infra::DfxArgsGetter;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gmod")]
#[command(
    version,
    about = "Galaxy Module Management Tool",
    long_about = "A comprehensive tool for managing Galaxy modules including creating new modules, updating existing ones, and localizing configurations."
)]
pub enum GxModCmd {
    /// Create example module structure
    #[command(
        about = "Create example module structure",
        long_about = "Create a complete example module structure with sample configurations and workflows to demonstrate module organization and best practices."
    )]
    Example,
    /// Define new module specification
    #[command(
        about = "Define new module operator ",
        long_about = "Create a new module specification with the given name. This will initialize a new module directory structure with all necessary configuration files."
    )]
    New(SpecArgs),
    /// Update existing module
    #[command(
        about = "Update existing module operator dependency",
        long_about = "Update an existing module's configuration, dependencies, or specifications. Supports force updates to override existing configurations."
    )]
    Update(UpdateArgs),
    /// Localize module configuration
    #[command(
        about = "Localize module configuration",
        long_about = "Generate localized configuration files for the module based on environment-specific values. Useful for adapting modules to different deployment environments."
    )]
    Localize(LocalArgs),
}

#[derive(Debug, Args, Getters)]
pub struct SpecArgs {
    /// Name of the new module to create
    #[arg(
        short,
        long,
        help = "Module name (alphanumeric with hyphens/underscores)"
    )]
    pub(crate) name: String,

    /// Enable debug output with specified level (0-3)
    #[arg(
        short = 'd',
        long = "debug",
        default_value = "0",
        help = "Debug level: 0=off, 1=basic, 2=verbose, 3=trace"
    )]
    pub debug: usize,
    /// Set logging level and format
    #[arg(long = "log", help = "Log level: error, warn, info, debug, trace")]
    pub log: Option<String>,
}
#[derive(Debug, Args, Getters)]
pub struct UpdateArgs {
    /// Enable debug output with specified level (0-3)
    #[arg(
        short = 'd',
        long = "debug",
        default_value = "0",
        help = "Debug level: 0=off, 1=basic, 2=verbose, 3=trace"
    )]
    pub debug: usize,
    /// Set logging level and format
    #[arg(long = "log", help = "Log level: error, warn, info, debug, trace")]
    pub log: Option<String>,

    /// Force update even if conflicts exist
    #[arg(
        short = 'f',
        long = "force",
        default_value = "0",
        help = "Force update: skip confirmation, overwrite existing files"
    )]
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
    /// Enable debug output with specified level (0-3)
    #[arg(
        short = 'd',
        long = "debug",
        default_value = "0",
        help = "Debug level: 0=off, 1=basic, 2=verbose, 3=trace"
    )]
    pub debug: usize,
    /// Set logging level and format
    #[arg(long = "log", help = "Log level: error, warn, info, debug, trace")]
    pub log: Option<String>,

    /// Path to values file for localization
    #[arg(
        long = "value",
        help = "Path to YAML/JSON file containing environment-specific values"
    )]
    pub value: Option<String>,
    /// Use default values instead of user-provided value.yml
    #[arg(long = "default", default_value = "false" , action = ArgAction::SetTrue, help = "Use default values instead of user-provided value.yml")]
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

impl DfxArgsGetter for SpecArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}
