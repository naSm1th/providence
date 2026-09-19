const WIRELESS_DEVICE_NAME: &str = "wlan0";

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

    // let session = iwdrs::session::Session::new().await.unwrap();
    // let device = match get_device_by_name(&session, WIRELESS_DEVICE_NAME).await {
    //     Ok(dev) => dev,
    //     Err(error) => return Err(error),
    // };

    // let adapter = device.adapter().await.unwrap();

    // println!(
    //     "Modes support by adapter {} (for device {}): {}",
    //     adapter.name().await.unwrap(),
    //     device.name().await.unwrap(),
    //     adapter.supported_modes().await.unwrap().join(", ")
    // );

    let iwd_handle =
        providence::network::iwd_wrapper::get_iwd_connection_by_name(WIRELESS_DEVICE_NAME).await?;
    providence::network::iwd_wrapper::print_device_info(&iwd_handle).await;

    // first, ensure the device is in AP mode
    match providence::network::iwd_wrapper::get_device_mode(&iwd_handle).await {
        Ok(iwdrs::modes::Mode::Ap) => println!("Device in AP mode already"),
        Ok(iwdrs::modes::Mode::Station) => {
            println!("Device in station mode. Switching to AP mode.");
            match providence::network::iwd_wrapper::set_device_mode(
                &iwd_handle,
                iwdrs::modes::Mode::Ap,
            )
            .await
            {
                Ok(()) => println!("Done."),
                _ => return Err("Failed to switch to AP mode.".to_string()),
            }
        }
        _ => return Err("Failed to retrieve device mode.".to_string()),
    }

    // second, ensure the device is powered
    match providence::network::iwd_wrapper::get_device_powered(&iwd_handle).await {
        Ok(true) => println!("Device already powered on."),
        Ok(false) => {
            println!("Device powered off. Powering on.");
            match providence::network::iwd_wrapper::set_device_powered(&iwd_handle, true).await {
                Ok(()) => println!("Done."),
                _ => return Err("Failed to power on device.".to_string()),
            }
        }
        _ => return Err("Failed to retrieve device power state.".to_string()),
    }

    // let session = iwdrs::session::Session::new().await.unwrap();
    let iwd_handle =
        providence::network::iwd_wrapper::get_iwd_connection_by_name(WIRELESS_DEVICE_NAME).await?;

    // now that the device is in AP mode, get the AP handle
    // do this in a loop because this seems to take a while
    let access_point = loop {
        let access_points = providence::network::iwd_wrapper::get_access_points(&iwd_handle).await?;
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

    // TODO: spawn a task to repeatedly scan and collect network list
    match access_point.scan().await {
        Ok(()) => println!("Scan started"),
        Err(_) => {
            shutdown().await.unwrap();
            return Err("Failed to start scan".to_string());
        }
    }

    while access_point.is_scanning().await.unwrap_or(true) {
        println!(".");
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    }

    if let Ok(networks) = access_point.networks().await {
        for network in networks {
            println!("{:?}", network);
        }
    }

    // wait for a button press to tear down and exit
    println!("\nPress Enter to stop AP and exit...");
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");

    return shutdown().await;
}
