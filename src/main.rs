//! Realtime seismometer monitor daemon which can execute programs and
//! publish topics to an MQTT server when certain events are detected.
use rs_udp::config::{Config, FlowConfig, SeismometerConfig};
use rs_udp::datasource::DataSource;
use rs_udp::overrides::{FlowTiedPath, SeismometerTiedPath};
use rs_udp::session::{ActionLoop, InstrumentLoop};
use rs_udp::session::{action_loop_message_channel, SensorFlow, MQTT};
use rs_udp::session::{AlarmSession, OutChannel};

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::{fs, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(name = env!("CARGO_BIN_NAME"))]
/// Real-time seismometer monitor
/// 
/// JSON Configuration Syntax:
/// 
/// Config = { 
///     "seismometers" : [ Seismometer+ ],
///     ( "mqtt" : MQTT )*
/// };
/// Seismometer = {
///     "name": string,
///     "listen": UDPListenSpec,
///     "sample_rate": number,
///     ( "timeout_s" : number )*,
///     ( "flows" : [ Flow* ] )*,
/// };
/// Flow = {
///     "name" : string,
///     "channel" : Channel,
///     "filter" : Filter,
///     "actions" : Actions,
/// };
/// Channel = "EHZ" | "EHN" | "EHE" | "ENZ" | "ENN" | "ENE";
/// Filter = {
///     ( "trigger_level" : number )*,
///     ( "reset_level" : number )*,
///     ( "offset" : number )*,
///     ( "gain" : number )*,
///     ( "order" : number )*,
///     ( "cutoff" : number )*,
///     ( "dc_alpha" : number )*,
///     ( "energy_alpha" : number )*,
///     ( "holdoff" : number )*,
/// };
/// Actions = {
///     ( "available_cmd" : string )*,
///     ( "unavailable_cmd" : string )*,
///     ( "trigger_cmd" : string )*,
///     ( "reset_cmd" : string )*,
///     ( "mqtt_topic": string )*,
///     ( "mqtt_available_topic" : string )*,
///     ( "mqtt_triggered_payload" : string )*,
///     ( "mqtt_reset_payload" : string )*,
///     ( "mqtt_available_payload" : string )*,
///     ( "mqtt_unavailable_payload" : string )*
/// };
/// MQTT = {
///     "host" : string,
///     ( "port" : number )*,
///     ( "client_id" : number )*,
///     ( "username" : number )*,
///     ( "password" : string )*,
/// };
pub struct Cli {
    /// Configuration file to use (JSON format)
    #[arg(short = 'c')]
    config_path: PathBuf,

    /// Supply data to a particular seismometer from a text file, masquerading
    /// as data from a specific seismometer channel.
    #[arg(short = 'f', value_names = [ "seismometer=channel:input-path"])]
    text_source: Vec<SeismometerTiedPath>,

    /// Dump filter process for a particular sensor to a file.
    #[arg(short = 'o', value_names = [ "flow=dump-path" ])]
    debug_output: Vec<FlowTiedPath>,
}

// Seismometer stream replacements by seismometer name.
type SeismometerRedirects<'a> = HashMap<&'a str, &'a SeismometerTiedPath>;
// Stream output inspections by flow name.
type FlowDumps<'a> = HashMap<&'a str, &'a FlowTiedPath>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = fs::read_to_string(&cli.config_path)
        .context("Failed to read config file")
        .and_then(|json| {
            serde_json::from_str::<Config>(&json).context("Failed to parse config JSON")
        })?;

    let session = configure_seismo_session(&cli, &config).await?;
    session.run().await?;

    Ok(())
}

// Configure an entire daemon session from command line arguments and
// configuration file.
async fn configure_seismo_session<'a>(
    cli: &'a Cli,
    config: &'a Config,
) -> Result<AlarmSession<'a>> {
    let source_overrides = redirects_by_seismometer(&cli.text_source);
    let dump_requests = dump_requests_by_flow_name(&cli.debug_output);
    let (tx_chan, rx_chan) = action_loop_message_channel();
    let MQTT(mqtt_client, mqtt_loop) = MQTT::from_config(config);
    let mut action_loop = ActionLoop::new(rx_chan, mqtt_client);
    let seismometer_loops =
        configure_seismometers_and_actions(config, &mut action_loop, tx_chan, source_overrides, dump_requests)
            .await?;

    let result = AlarmSession::new(
        seismometer_loops,
        action_loop,
        mqtt_loop,
    );
    Ok(result)
}

// Build a list of instruments to monitor and the actions to take when they
// are triggered from a configuration template and a set of overrides for
// data stream injection an data dump outputs.
//
// This function borrows items from the Configuration structure, so its
// outputs are only valid for the Configuration's lifetime.
async fn configure_seismometers_and_actions<'a>(
    config: &'a Config,
    action_loop: &mut ActionLoop<'a>,
    action_channel: OutChannel,
    source_overrides: SeismometerRedirects<'a>,
    dump_requests: FlowDumps<'a>,
) -> Result<Vec<InstrumentLoop>, anyhow::Error> {
    let mut loops: Vec<InstrumentLoop> = Vec::new();
    let mut flow_id: usize = 0;

    for seismometer_config in config.seismometers.iter() {
        let mut instrument = instrument_loop_from_config_and_overrides(
            seismometer_config,
            &action_channel,
            &source_overrides,
        )
        .await?;
        for flow_config in seismometer_config.flows.iter() {
            let flow = flow_from_config_and_dump_requests(
                seismometer_config.sample_rate,
                flow_config,
                &dump_requests,
            )
            .await?;
            instrument.add_flow(flow_id, flow_config.channel.as_str().try_into()?, flow);
            action_loop.add_flow(flow_id, &flow_config.name, &flow_config.actions);
            flow_id += 1;
        }
        loops.push(instrument);
    }
    Ok(loops)
}

async fn instrument_loop_from_config_and_overrides(
    seismometer_config: &SeismometerConfig,
    action_channel: &OutChannel,
    source_overrides: &SeismometerRedirects<'_>,
) -> Result<InstrumentLoop> {
    let source = datasource_for_seismometer(seismometer_config, source_overrides).await?;
    let iloop = InstrumentLoop::new_for_datasource(
        source,
        seismometer_config.timeout_s,
        action_channel.clone(),
    );
    Ok(iloop)
}

// Set up a signal flow processor, with an option to inspect its flow and
// dump diagnostics to a text file.
async fn flow_from_config_and_dump_requests(
    sample_rate_hz: f32,
    config: &FlowConfig,
    dump_requests: &FlowDumps<'_>,
) -> Result<SensorFlow> {
    let dump_request = dump_requests.get(config.name.as_str()).map(|x| &x.path);
    let flow = SensorFlow::from_config(sample_rate_hz, config, dump_request).await?;
    Ok(flow)
}

// Set up a data source for a particular seismometer, allowing for it to be
// overriden from the command line.
async fn datasource_for_seismometer(
    config: &SeismometerConfig,
    overrides: &SeismometerRedirects<'_>,
) -> Result<DataSource> {
    Ok(match overrides.get(config.name.as_str()) {
        Some(&path) => DataSource::new_textfile_source(&path.path, path.channel).await?,
        None => DataSource::new_rsudp_source(&config.listen).await?,
    })
}
//}

/// Build a quick lookup table to query whether a seismometer should be
/// "faked" by data from a text file.
fn redirects_by_seismometer(specs: &[SeismometerTiedPath]) -> SeismometerRedirects {
    specs
        .iter()
        .map(|x| (x.seismometer_name.as_str(), x))
        .collect()
}

/// Build a quick lookup table to query whether a filesystem path has been
/// associated with a flow output by the user.
fn dump_requests_by_flow_name(specs: &[FlowTiedPath]) -> FlowDumps {
    specs
        .iter()
        .map(|spec| (spec.flow_name.as_str(), spec))
        .collect()
}
