// This file contains high-level network operations. It acts as a wrapper around lower-level
// operations contained in ap.rs (access point operations) and client.rs (wifi client operations).

mod ap;
mod client;
pub mod iwd_wrapper;

use crate::configuration;

enum Error {
    InvalidConfiguration,
    HardwareFailure,
}

enum Mode {
    Idle,
    AccessPoint,
    Client,
}

struct NetworkResult {
    network_name: String,
    network_security: configuration::NetworkSecurityConfig,
    signal_strength: u32,
}

// attempt to start in client mode
// returns:
// - success - connected
// - success - not yet connected
// - error - no configuration/invalid
async fn client_start(config: configuration::WifiClientConfig) -> Result<(), Error> {
    Ok(())
}

// shutdown client
async fn client_shutdown() -> Result<(), Error> {
    Ok(())
}

// attempt to start in access point mode
// returns:
// - success - AP started
// - error - hardware failure?
async fn ap_start(config: configuration::WifiApConfig) -> Result<(), Error> {
    Ok(())
}

// shutdown access point
async fn ap_shutdown() -> Result<(), Error> {
    Ok(())
}

// scan for network (async blocks)
async fn ap_scan_for_networks() -> Result<Vec<NetworkResult>, Error> {
    Ok(vec![])
}

// attempt connection to network
// returns:
// - success - connected to network
// - error - failed to connect
async fn attempt_connection(network: configuration::WifiClientConfig) -> Result<(), Error> {
    Ok(())
}

// get current operating mode and state
async fn get_mode() -> Result<Mode, Error> {
    Ok(Mode::Idle)
}
