// This file contains high-level network operations. It acts as a wrapper around lower-level
// operations contained in ap.rs (access point operations) and client.rs (wifi client operations).

mod ap;
mod client;

use crate::configuration;

enum Error {
    InvalidConfiguration,
    HardwareFailure,
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

// attempt connection to network
// returns:
// - success - connected to network
// - error - failed to connect

// get current operating mode and state
