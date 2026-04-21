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
    interface = "net.connman.iwd.Device",
    default_service = "net.connman.iwd"
)]
pub trait Device {
    /// Adapter property
    #[zbus(property)]
    fn adapter(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Address property
    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;

    /// Mode property
    #[zbus(property)]
    fn mode(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_mode(&self, value: &str) -> zbus::Result<()>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// Powered property
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_powered(&self, value: bool) -> zbus::Result<()>;
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
    let adapter_interface_name = OwnedInterfaceName::from(InterfaceName::try_from("net.connman.iwd.Adapter").unwrap());
    let station_interface_name =
        OwnedInterfaceName::from(InterfaceName::try_from("net.connman.iwd.Station").unwrap());
    let ap_interface_name =
        OwnedInterfaceName::from(InterfaceName::try_from("net.connman.iwd.AccessPoint").unwrap());

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
            let device_path = object.0.clone();
            let device_proxy = DeviceProxy::new(&connection, &device_path).await?;
            let station_proxy = StationProxy::new(&connection, &device_path).await?;
            let adapter_proxy: AdapterProxy = AdapterProxy::new(&connection, device_proxy.adapter().await.unwrap()).await?;

            println!("Device:");
            println!("\tPath: {}", device_path);

            println!(
                "\tName: {}",
                device_proxy.name().await.unwrap()
            );
            // println!(
            //     "\tState: {}",
            //     station_proxy.state().await.unwrap()
            // );
            // println!(
            //     "\tScanning: {}",
            //     station_proxy.scanning().await.unwrap()
            // );
            println!(
                "\tMode: {}",
                device_proxy.mode().await.unwrap()
            );
            println!(
                "\tPowered: {}",
                device_proxy.powered().await.unwrap()
            );
            println!("\tModes supported: {:?}",
                adapter_proxy.supported_modes().await.unwrap_or(Vec::<String>::new())
            );
            println!("\tCurrent mode: {}", device_proxy.mode().await.unwrap());
            println!("\n\n");

            println!("Setting mode to ap");
            if station_proxy.disconnect().await.is_err() {
                println!("Failed to disconnect");
            }
            if device_proxy.set_mode("ap").await.is_err() {
                println!("Failed to set mode");
            }
            // while true {
            //     println!("New mode: {}", device_proxy.mode().await.unwrap());
            //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            // }
            println!("\n\n");

            let networks_list = station_proxy.get_ordered_networks().await?;
            // let networks_list = ap_proxy.get_ordered_networks().await?;

            println!("| Connected? | SSID                             | Type  | Signal  |");
            println!("|------------|----------------------------------|-------|---------|");

            for network in networks_list {
                let network_proxy = NetworkProxy::new(&connection, network.0.as_str()).await?;

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
