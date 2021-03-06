//! GozScoring Abscissa Application

use crate::prelude::*;
use crate::{commands::GozScoringCmd, config::GozScoringConfig};
use abscissa_core::{
    application::{self, AppCell},
    config, trace, Application, EntryPoint, FrameworkError, StandardPaths,
};
use relayer_modules::events::IBCEvent;
use sagan::message::Envelope;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use subtle_encoding::bech32::{decode, encode};

/// Application state
pub static APPLICATION: AppCell<GozScoringApp> = AppCell::new();

/// Obtain a read-only (multi-reader) lock on the application state.
///
/// Panics if the application state has not been initialized.
pub fn app_reader() -> application::lock::Reader<GozScoringApp> {
    APPLICATION.read()
}

/// Obtain an exclusive mutable lock on the application state.
pub fn app_writer() -> application::lock::Writer<GozScoringApp> {
    APPLICATION.write()
}

/// Obtain a read-only (multi-reader) lock on the application configuration.
///
/// Panics if the application configuration has not been loaded.
pub fn app_config() -> config::Reader<GozScoringApp> {
    config::Reader::new(&APPLICATION)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Score {
    pub hub_opaque_packets: u64,
    pub opaque_packets_tx: u64,
    pub packets_from_hub: u64,
    pub opaque_packets_total: u64,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            hub_opaque_packets: 0,
            opaque_packets_tx: 0,
            packets_from_hub: 0,
            opaque_packets_total: 0,
        }
    }
}
/// GozScoring Application
#[derive(Debug)]
pub struct GozScoringApp {
    /// Application configuration.
    config: Option<GozScoringConfig>,

    /// Score for each team
    scores: HashMap<String, Score>,

    /// Application state.
    state: application::State<GozScoringApp>,

    /// Hashmap from Address to team
    address_to_team: HashMap<String, String>,

    ///Source channels on the Hub
    source_channels: BTreeSet<String>,

    ///Source channels on the Hub
    observed_transactions: BTreeSet<String>,
}

impl fmt::Display for GozScoringApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (team, score) in self.scores.iter() {
            let total_score = (score.hub_opaque_packets as f64)
                + (score.packets_from_hub as f64 * 0.5)
                + (score.opaque_packets_tx as f64 * 0.1);

            write!(
                f,
                "Team:{}, Total Phase 2 Score {}, Total Packets Relayed{})\n",
                team, total_score, score.opaque_packets_total
            )?;
        }

        Ok(())
    }
}

impl GozScoringApp {
    pub fn print(&self) {
        let mut buf = Vec::new();
        for (team, score) in self.scores.iter() {
            let total_score = (score.hub_opaque_packets as f64)
                + (score.packets_from_hub as f64 * 0.5)
                + (score.opaque_packets_tx as f64 * 0.1);

            write!(
                &mut buf,
                "Team:{}, Total Phase 2 Score {}, Total Packets Relayed {})\n",
                team, total_score, score.opaque_packets_total
            )
            .unwrap();
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open("results.txt")
            .unwrap();
        file.write_all(&buf).unwrap();
    }

    pub fn score_envelope(&mut self, envelope: Envelope) {
        for message in envelope.msg {
            // status_ok!("Running", "Processing Message {:?}", message );

            match message {
                sagan::message::Message::EventIBC(ref event) => {
                    match event {
                        /// Compute all the scoring for an opaque packet
                        IBCEvent::OpaquePacket(ref inner_event) => {
                            status_ok!("Processing oqaque packet", " got event");
                            if let Some(tx_hashes) = inner_event.data.get("tx.hash") {
                                if let Some(hash) = tx_hashes.get(0) {
                                    /// Don't double count packets
                                    if !self.observed_transactions.contains(hash) {
                                        status_ok!("Processing oqaque packet", " Hash Unknown");

                                        self.observed_transactions.insert(hash.clone());

                                        if let Some(senders) =
                                            inner_event.data.get("message.sender")
                                        {
                                            status_ok!("Processing oqaque packet", "Got Senders");

                                            if let Some(src_channels) = inner_event
                                                .data
                                                .get("recv_packet.packet_src_channel")
                                            {
                                                // Get the second to last sender to use to assign a packet to a team
                                                if let Some(sender_address) =
                                                    senders.get(src_channels.len() + 1)
                                                {
                                                    status_ok!(
                                                        "Processing oqaque packet",
                                                        "Got Sender"
                                                    );

                                                    if let Some(team) =
                                                        self.get_team_by_address(sender_address)
                                                    {
                                                        status_ok!(
                                                            "Processing oqaque packet",
                                                            "Scoring"
                                                        );

                                                        let score = self
                                                            .scores
                                                            .entry(team.clone())
                                                            .or_insert(Score::default());

                                                        if let Some(config) = &self.config {
                                                            if let Some(channel) =
                                                                src_channels.get(0)
                                                            {
                                                                if config.hub_id.contains(
                                                                    &envelope.network.to_string(),
                                                                ) {
                                                                    score.hub_opaque_packets += 1;
                                                                } else if self
                                                                    .source_channels
                                                                    .contains(channel)
                                                                {
                                                                    score.packets_from_hub += 1;
                                                                } else {
                                                                    score.opaque_packets_tx += 1;
                                                                }

                                                                // Use src channels as proxy for the number of packets in a multimessage
                                                                score.opaque_packets_total +=
                                                                    src_channels.len() as u64;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        IBCEvent::PacketTransfer(ref inner_event) => {
                            status_ok!("Processing Packet Transfer", " got event");

                            if let Some(config) = &self.config {
                                if config.hub_id.contains(&envelope.network.to_string()) {
                                    if let Some(dst_channels) =
                                        inner_event.data.get("send_packet.packet_dst_channel")
                                    {
                                        for dst_channel in dst_channels {
                                            /// Populate the source channels data
                                            self.source_channels.insert(dst_channel.clone());
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }

    fn get_team_by_address(&self, address: &str) -> Option<&String> {
        if address.contains("cosmos1") {
            return self.address_to_team.get(address);
        } else {
            match decode(address) {
                Ok((_, data)) => return self.address_to_team.get(&encode("cosmos", data)),
                Err(_) => return None,
            }
        }
    }
}

/// Initialize a new application instance.
///
/// By default no configuration is loaded, and the framework state is
/// initialized to a default, empty state (no components, threads, etc).
impl Default for GozScoringApp {
    fn default() -> Self {
        Self {
            config: None,
            scores: HashMap::new(),
            state: application::State::default(),
            address_to_team: HashMap::new(),
            source_channels: BTreeSet::new(),
            observed_transactions: BTreeSet::new(),
        }
    }
}

impl Application for GozScoringApp {
    /// Entrypoint command for this application.
    type Cmd = EntryPoint<GozScoringCmd>;

    /// Application configuration.
    type Cfg = GozScoringConfig;

    /// Paths to resources within the application.
    type Paths = StandardPaths;

    /// Accessor for application configuration.
    fn config(&self) -> &GozScoringConfig {
        self.config.as_ref().expect("config not loaded")
    }

    /// Borrow the application state immutably.
    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    /// Borrow the application state mutably.
    fn state_mut(&mut self) -> &mut application::State<Self> {
        &mut self.state
    }

    /// Register all components used by this application.
    ///
    /// If you would like to add additional components to your application
    /// beyond the default ones provided by the framework, this is the place
    /// to do so.
    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let components = self.framework_components(command)?;
        self.state.components.register(components)
    }

    /// Post-configuration lifecycle callback.
    ///
    /// Called regardless of whether config is loaded to indicate this is the
    /// time in app lifecycle when configuration would be loaded if
    /// possible.
    fn after_config(&mut self, config: Self::Cfg) -> Result<(), FrameworkError> {
        // Configure components
        self.state.components.after_config(&config)?;
        status_ok!("Config", "Build Hashmaps");
        self.address_to_team = config.build_hashmaps();
        dbg!(&self.address_to_team);

        self.config = Some(config);

        Ok(())
    }

    /// Get tracing configuration from command-line options
    fn tracing_config(&self, command: &EntryPoint<GozScoringCmd>) -> trace::Config {
        if command.verbose {
            trace::Config::verbose()
        } else {
            trace::Config::default()
        }
    }
}
