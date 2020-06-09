//! `start` subcommand - example of how to write a subcommand

use crate::application::APPLICATION;
/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use crate::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::fmt;

use sagan::message::Envelope;

use crate::config::GozScoringConfig;
use abscissa_core::{config, Command, FrameworkError, Options, Runnable};
use std::path::PathBuf;

/// `start` subcommand
///
/// The `Options` proc macro generates an option parser based on the struct
/// definition, and is defined in the `gumdrop` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/gumdrop/>
#[derive(Command, Debug, Options)]
pub struct StartCmd {
    /// To whom are we saying hello?
    #[options(free)]
    event_jsons: Vec<PathBuf>,
}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        let config = app_config();

        for scoreable in self.event_jsons.as_slice() {
            let f = File::open(scoreable).expect(&format!("Could not open file {:?}", scoreable));
            let f = BufReader::new(f);
            for line in f.lines() {
                let line = line.expect("Failed to read line");

                match serde_json::from_str(&line) {
                    Ok(envelope) => {
                        status_ok!("Running","processing envelope");

                       let mut state =app_writer();

                       state.score_envelope(envelope);
                    }
                    Err(e) => status_err!("Could not parse json {}", e),
                }
            }
        }
       APPLICATION.read().print();
    }
}

// impl config::Override<GozScoringConfig> for StartCmd {
//     // Process the given command line options, overriding settings from
//     // a configuration file using explicit flags taken from command-line
//     // arguments.
//     fn override_config(
//         &self,
//         mut config: GozScoringConfig,
//     ) -> Result<GozScoringConfig, FrameworkError> {
//         if !self.recipient.is_empty() {
//             config.hello.recipient = self.recipient.join(" ");
//         }

//         Ok(config)
//     }
// }
