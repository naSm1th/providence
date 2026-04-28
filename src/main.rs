const WIRELESS_DEVICE_NAME: &str = "wlan0";

#[tokio::main]
async fn main() -> Result<(), String> {
    let session = iwdrs::session::Session::new().await.unwrap();
    let devices: Vec<iwdrs::device::Device> = session.devices().await.unwrap();
    let device_option: std::option::Option<&iwdrs::device::Device> = futures::future::join_all(
        devices
            .iter()
            .map(|dev: &iwdrs::device::Device| async move {
                (dev, dev.name().await.unwrap().eq(WIRELESS_DEVICE_NAME))
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
        _ => return Err(format!("Could not find device {}", WIRELESS_DEVICE_NAME)),
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


    // now that the device is in AP mode, get the AP handle
    // do this in a loop because this seems to take a while
    let access_point = loop {
        let access_points = session.access_points().await.unwrap();
        match access_points.first() {
            Some(ap) => break ap.clone(),
            _ => println!("."),
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    };

    match access_point.start("bananya", "nyanyanya").await {
        Ok(()) => println!("AP started."),
        Err(_) => return Err("Failed to start AP".to_string()),
    }

    // tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;

    // wait for a button press to tear down and exit
    println!("\nPress Enter to stop AP and exit...");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).expect("Failed to read line");

    // we must tear down the AP when we are done
    match access_point.stop().await {
        Ok(()) => println!("AP stopped."),
        Err(_) => return Err("Failed to stop AP".to_string()),
    }

    Ok(())
}
