//! GozScoring Subcommands
//!
//! This is where you specify the subcommands of your application.
//!
//! The default application comes with two subcommands:
//!
//! - `start`: launches the application
//! - `version`: print application version
//!
//! See the `impl Configurable` below for how to specify the path to the
//! application's configuration file.

mod start;
mod version;

use self::{start::StartCmd, version::VersionCmd};
use crate::config::GozScoringConfig;
use abscissa_core::{
    config::Override, Command, Configurable, FrameworkError, Help, Options, Runnable,
};
use std::path::PathBuf;

/// GozScoring Configuration Filename
pub const CONFIG_FILE: &str = "goz_scoring.toml";

/// GozScoring Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum GozScoringCmd {
    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `start` subcommand
    #[options(help = "start the application")]
    Start(StartCmd),

    /// The `version` subcommand
    #[options(help = "display version information")]
    Version(VersionCmd),
}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<GozScoringConfig> for GozScoringCmd {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        // Check if the config file exists, and if it does not, ignore it.
        // If you'd like for a missing configuration file to be a hard error
        // instead, always return `Some(CONFIG_FILE)` here.
        let filename = PathBuf::from(CONFIG_FILE);

        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    }

    // /// Apply changes to the config after it's been loaded, e.g. overriding
    // /// values in a config file using command-line options.
    // ///
    // /// This can be safely deleted if you don't want to override config
    // /// settings from command-line options.
    // fn process_config(
    //     &self,
    //     config: GozScoringConfig,
    // ) -> Result<GozScoringConfig, FrameworkError> {
    //     match self {
    //         GozScoringCmd::Start(cmd) => cmd.override_config(config),
    //         _ => Ok(config),
    //     }
    // }
}
