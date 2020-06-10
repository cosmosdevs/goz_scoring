//! GozScoring Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use sagan::config::collector::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GozScoring Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GozScoringConfig {
    /// Chain-id of the Hub
    pub hub_id: Vec<String>,
    /// An example configuration section
    pub teams: Vec<Team>,
}

impl GozScoringConfig {
    pub fn build_hashmaps(&self) -> HashMap<String, String> {
        let mut address_to_team = HashMap::new();

        for team in self.teams.as_slice() {
            address_to_team.insert(
                team.address.to_lowercase().replace(" ", "_").clone(),
                team.name.clone(),
            );
        }

        return address_to_team;
    }
}

/// Default configuration settings.
///
/// Note: if your needs are as simple as below, you can
/// use `#[derive(Default)]` on GozScoringConfig instead.
impl Default for GozScoringConfig {
    fn default() -> Self {
        Self {
            hub_id: Vec::new(),
            teams: Vec::new(),
        }
    }
}
