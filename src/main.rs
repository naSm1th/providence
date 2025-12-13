use std::{error::Error, ops::Deref};

use zbus::{
    Connection,
    fdo::ObjectManagerProxy,
    names::{InterfaceName, OwnedInterfaceName},
    proxy, zvariant,
};

#[proxy(
    interface = "net.connman.iwd.Station",
    default_service = "net.connman.iwd"
)]
pub trait Station {
    /// ConnectHiddenNetwork method
    fn connect_hidden_network(&self, name: &str) -> zbus::Result<()>;

    /// Disconnect method
    fn disconnect(&self) -> zbus::Result<()>;

    /// GetHiddenAccessPoints method
    fn get_hidden_access_points(&self) -> zbus::Result<Vec<(String, i16, String)>>;

    /// GetOrderedNetworks method
    fn get_ordered_networks(&self) -> zbus::Result<Vec<(zbus::zvariant::OwnedObjectPath, i16)>>;

    /// RegisterSignalLevelAgent method
    fn register_signal_level_agent(
        &self,
        path: &zbus::zvariant::ObjectPath<'_>,
        levels: &[i16],
    ) -> zbus::Result<()>;

    /// Scan method
    fn scan(&self) -> zbus::Result<()>;

    /// UnregisterSignalLevelAgent method
    fn unregister_signal_level_agent(
        &self,
        path: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// ConnectedNetwork property
    #[zbus(property)]
    fn connected_network(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Scanning property
    #[zbus(property)]
    fn scanning(&self) -> zbus::Result<bool>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<String>;
}

#[proxy(
    interface = "net.connman.iwd.Adapter",
    default_service = "net.connman.iwd"
)]
pub trait Adapter {
    /// Model property
    #[zbus(property)]
    fn model(&self) -> zbus::Result<String>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// Powered property
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_powered(&self, value: bool) -> zbus::Result<()>;

    /// SupportedModes property
    #[zbus(property)]
    fn supported_modes(&self) -> zbus::Result<Vec<String>>;

    /// Vendor property
    #[zbus(property)]
    fn vendor(&self) -> zbus::Result<String>;
}

#[proxy(
    interface = "net.connman.iwd.p2p.Device",
    default_service = "net.connman.iwd"
)]
pub trait Device {
    /// GetPeers method
    fn get_peers(&self) -> zbus::Result<Vec<(zbus::zvariant::OwnedObjectPath, i16)>>;

    /// ReleaseDiscovery method
    fn release_discovery(&self) -> zbus::Result<()>;

    /// RequestDiscovery method
    fn request_discovery(&self) -> zbus::Result<()>;

    /// AvailableConnections property
    #[zbus(property)]
    fn available_connections(&self) -> zbus::Result<u16>;

    /// Enabled property
    #[zbus(property)]
    fn enabled(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_enabled(&self, value: bool) -> zbus::Result<()>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_name(&self, value: &str) -> zbus::Result<()>;
}

#[proxy(
    interface = "net.connman.iwd.Network",
    default_service = "net.connman.iwd"
)]
pub trait Network {
    /// Connect method
    fn connect(&self) -> zbus::Result<()>;

    /// Connected property
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    /// Device property
    #[zbus(property)]
    fn device(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// KnownNetwork property
    #[zbus(property)]
    fn known_network(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// Type property
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;
}

// TODO: make a function that converts Adapters and Devices into suitable structs

const WIRELESS_DEVICE_NAME: &str = "wlan0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let proxy = ObjectManagerProxy::new(&connection, "net.connman.iwd", "/").await?;
    let managed_objects = proxy.get_managed_objects().await?;
    let device_interface_name =
        OwnedInterfaceName::from(InterfaceName::try_from("net.connman.iwd.Device").unwrap());
    let station_interface_name =
        OwnedInterfaceName::from(InterfaceName::try_from("net.connman.iwd.Station").unwrap());

    for object in managed_objects {
        if object.1.contains_key(&device_interface_name)
            && object
                .1
                .get(&device_interface_name)
                .unwrap()
                .get("Name")
                .unwrap()
                .deref()
                .clone()
                .downcast::<zvariant::Str>()
                .unwrap()
                .as_str()
                == WIRELESS_DEVICE_NAME
        {
            println!("Device:");
            println!("\tPath: {}", object.0);

            let device_object = object.1.get(&device_interface_name).unwrap();
            let station_object = object.1.get(&station_interface_name).unwrap();

            println!(
                "\tName: {}",
                device_object
                    .get("Name")
                    .unwrap()
                    .deref()
                    .clone()
                    .downcast::<zvariant::Str>()
                    .unwrap()
                    .as_str()
            );
            println!(
                "\tState: {}",
                station_object
                    .get("State")
                    .unwrap()
                    .deref()
                    .clone()
                    .downcast::<zvariant::Str>()
                    .unwrap()
                    .as_str()
            );
            println!(
                "\tScanning: {}",
                station_object
                    .get("Scanning")
                    .unwrap()
                    .deref()
                    .clone()
                    .downcast::<bool>()
                    .unwrap()
            );
            println!(
                "\tMode: {}",
                device_object
                    .get("Mode")
                    .unwrap()
                    .deref()
                    .clone()
                    .downcast::<zvariant::Str>()
                    .unwrap()
                    .as_str()
            );
            println!(
                "\tPowered: {}",
                device_object
                    .get("Powered")
                    .unwrap()
                    .deref()
                    .clone()
                    .downcast::<bool>()
                    .unwrap()
            );
            println!("\n\n");

            let station_proxy: StationProxy =
                StationProxy::new(&connection, object.0.as_str()).await?;
            let networks_list = station_proxy.get_ordered_networks().await?;

            // print header
            println!("| Connected? | SSID                             | Type  | Signal  |");
            println!("|------------|----------------------------------|-------|---------|");

            for network in networks_list {
                let network_proxy = NetworkProxy::new(&connection, network.0.as_str()).await?;
                // println!(
                //     "Connected: {}",
                //     network_proxy
                //         .connected()
                //         .await
                //         .map_or(String::from("Error getting value"), |value| value
                //             .to_string())
                // );
                // println!(
                //     "Name: {}",
                //     network_proxy
                //         .name()
                //         .await
                //         .unwrap_or(String::from("Error getting value"))
                // );
                // println!(
                //     "Type: {}",
                //     network_proxy
                //         .type_()
                //         .await
                //         .unwrap_or(String::from("Error getting value"))
                // );
                // println!("Signal: {}dbm", network.1 / 100);
                // println!();

                println!(
                    "| {:10} | {:32} | {:5} | {:7} |",
                    network_proxy
                        .connected()
                        .await
                        .map_or(String::from("Error getting value"), |value| value
                            .to_string()),
                    network_proxy
                        .name()
                        .await
                        .unwrap_or(String::from("Error getting value")),
                    network_proxy
                        .type_()
                        .await
                        .unwrap_or(String::from("Error getting value")),
                    network.1 / 100
                );
            }
        }
    }

    Ok(())
}
