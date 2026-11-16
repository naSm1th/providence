use futures::TryStreamExt;
use rtnetlink::AddressMessageBuilder;

const WIRELESS_DEVICE_NAME: &str = "wlan0";

async fn get_device_by_name(
    session: &iwdrs::session::Session,
    name: impl Into<String>,
) -> Result<iwdrs::device::Device, String> {
    let name_string = name.into();
    let devices: Vec<iwdrs::device::Device> = session.devices().await.unwrap();
    let device_option: std::option::Option<&iwdrs::device::Device> = futures::future::join_all(
        devices.iter().map(|dev: &iwdrs::device::Device| {
            let value = name_string.clone();
            async move { (dev, dev.name().await.unwrap().eq(&value)) }
        }), // .collect(),
    )
    .await
    .into_iter()
    .filter_map(|(dev, valid)| if valid { Some(dev) } else { None })
    .collect::<Vec<&iwdrs::device::Device>>()
    .first()
    .map(|dev| &**dev);

    let device = match device_option {
        Some(device) => device,
        _ => return Err(format!("Could not find device {}", &name_string)),
    };

    Ok(device.clone())
}

async fn shutdown() -> Result<(), String> {
    // we must tear down the AP when we are done
    // we must get the AP from the session, which must be retrieved first
    let session = iwdrs::session::Session::new().await.unwrap();
    let access_points = session.access_points().await.unwrap();
    let access_point = match access_points.first() {
        Some(ap) => ap,
        _ => return Err("Failed to find AP".to_string()),
    };

    match access_point.stop().await {
        Ok(()) => println!("AP stopped."),
        Err(_) => return Err("Failed to stop AP".to_string()),
    }

    flush_ip_addrs_on_iface(WIRELESS_DEVICE_NAME).await
}

async fn set_ip_addr_on_iface(
    iface_name: impl Into<String>,
    ip_addr: std::net::IpAddr,
    netmask: u8,
) -> Result<(), String> {
    let iface_name_string = iface_name.into();
    let (connection, handle, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(connection);
    let mut links = handle.link().get().match_name(&iface_name_string).execute();
    if let Some(link) = links.try_next().await.unwrap() {
        handle
            .address()
            .add(link.header.index, ip_addr, netmask)
            .execute()
            .await
            .map_err(|_| format!("Failed to set ip address on iface {}", iface_name_string))?;
    } else {
        return Err(format!("iface with name {} not found", iface_name_string));
    }

    Ok(())
}

async fn flush_ip_addrs_on_iface(iface_name: impl Into<String>) -> Result<(), String> {
    // drop the IP addresses
    let (connection, handle, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(connection);
    let mut links = handle.link().get().match_name(iface_name).execute();
    if let Some(link) = links.try_next().await.unwrap() {
        let mut addresses = handle
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute();
        while let Some(msg) = addresses.try_next().await.unwrap() {
            // println!("{0:?}", msg.attributes);

            if let Some(address) = msg.attributes.iter().find_map(|item| match item {
                rtnetlink::packet_route::address::AddressAttribute::Address(addr) => Some(*addr),
                _ => None,
            }) {
                let message = match address {
                    std::net::IpAddr::V4(addr) => {
                        AddressMessageBuilder::<std::net::Ipv4Addr>::new()
                            .index(link.header.index)
                            .address(addr, msg.header.prefix_len)
                            .build()
                    }
                    std::net::IpAddr::V6(addr) => {
                        AddressMessageBuilder::<std::net::Ipv6Addr>::new()
                            .index(link.header.index)
                            .address(addr, msg.header.prefix_len)
                            .build()
                    }
                };
                handle.address().del(message).execute().await.unwrap()
            };
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    ctrlc::set_handler(move || {
        let new_rt = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime for Ctrl-C handler");
        let res: Result<(), String> = new_rt.block_on(shutdown());
        std::process::exit(if res.is_ok() { 0 } else { 1 });
    })
    .expect("Error setting Ctrl-C handler");

    let session = iwdrs::session::Session::new().await.unwrap();
    let device = match get_device_by_name(&session, WIRELESS_DEVICE_NAME).await {
        Ok(dev) => dev,
        Err(error) => return Err(error),
    };

    let adapter = device.adapter().await.unwrap();

    println!(
        "Modes support by adapter {} (for device {}): {}",
        adapter.name().await.unwrap(),
        device.name().await.unwrap(),
        adapter.supported_modes().await.unwrap().join(", ")
    );

    // first, ensure the device is in AP mode
    match device.get_mode().await {
        Ok(iwdrs::modes::Mode::Ap) => println!("Device in AP mode already"),
        Ok(iwdrs::modes::Mode::Station) => {
            println!("Device in station mode. Switching to AP mode.");
            match device.set_mode(iwdrs::modes::Mode::Ap).await {
                Ok(()) => println!("Done."),
                _ => return Err("Failed to switch to AP mode.".to_string()),
            }
        }
        _ => return Err("Failed to retrieve device mode.".to_string()),
    }

    // second, ensure the device is powered
    match device.is_powered().await {
        Ok(true) => println!("Device already powered on."),
        Ok(false) => {
            println!("Device powered off. Powering on.");
            match device.set_power(true).await {
                Ok(()) => println!("Done."),
                _ => return Err("Failed to power on device.".to_string()),
            }
        }
        _ => return Err("Failed to retrieve device power state.".to_string()),
    }

    let session = iwdrs::session::Session::new().await.unwrap();

    // now that the device is in AP mode, get the AP handle
    // do this in a loop because this seems to take a while
    let access_point = loop {
        let access_points = session.access_points().await.unwrap();
        match access_points.first() {
            Some(ap) => break ap.clone(),
            _ => println!("."),
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    };

    match access_point.start("bananya", "nyanyanya").await {
        Ok(()) => println!("AP started."),
        Err(_) => return Err("Failed to start AP".to_string()),
    }

    // tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;

    set_ip_addr_on_iface(
        WIRELESS_DEVICE_NAME,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 120, 2)),
        24,
    )
    .await?;

    // wait for a button press to tear down and exit
    println!("\nPress Enter to stop AP and exit...");
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");

    return shutdown().await;
}
