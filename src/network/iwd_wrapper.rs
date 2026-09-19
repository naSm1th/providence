// This file containers helper functions that wrap iwd_rs for convenience.

pub struct IwdHandle {
    iwd_session: iwdrs::session::Session,
    device: iwdrs::device::Device,
    adapter: iwdrs::adapter::Adapter,
    access_point: Option<iwdrs::access_point::AccessPoint>,
}

pub async fn get_iwd_connection_by_name(device_name: &str) -> Result<IwdHandle, String> {
    let iwd_session = match iwdrs::session::Session::new().await {
        Ok(session) => Ok(session),
        _ => Err("Could not get IWD session".to_string()),
    }?;
    let device = get_device_by_name(&iwd_session, device_name).await?;
    let adapter = get_adapter_from_device(&device).await?;

    Ok(IwdHandle {
        iwd_session,
        device,
        adapter,
        access_point: None
    })
}

pub async fn print_device_info(handle: &IwdHandle) {
    println!(
        "Modes support by adapter {} (for device {}): {}",
        handle.adapter.name().await.unwrap(),
        handle.device.name().await.unwrap(),
        handle.adapter.supported_modes().await.unwrap().join(", ")
    );
}

async fn get_device_by_name(
    session: &iwdrs::session::Session,
    name: &str,
) -> Result<iwdrs::device::Device, String> {
    let devices: Vec<iwdrs::device::Device> = session.devices().await.unwrap();
    let device_option: std::option::Option<&iwdrs::device::Device> =
        futures::future::join_all(
            devices
                .iter()
                .map(|dev: &iwdrs::device::Device| async move {
                    (dev, dev.name().await.unwrap().eq(name))
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
        _ => return Err(format!("Could not find device {}", name)),
    };

    Ok(device.clone())
}

async fn get_adapter_from_device(
    device: &iwdrs::device::Device,
) -> Result<iwdrs::adapter::Adapter, String> {
    match device.adapter().await {
        Ok(adapter) => Ok(adapter),
        _ => Err("Could not get adapter".to_string()),
    }
}

pub async fn get_device_mode(handle: &IwdHandle) -> Result<iwdrs::modes::Mode, String> {
    match handle.device.get_mode().await {
        Ok(mode) => Ok(mode),
        _ => Err("Could not retrieve device mode.".to_string()),
    }
}

pub async fn set_device_mode(handle: &IwdHandle, mode: iwdrs::modes::Mode) -> Result<(), String> {
    handle
        .device
        .set_mode(mode)
        .await
        .map_err(|_| "Could not switch to AP mode.".to_string())
}

pub async fn get_device_powered(handle: &IwdHandle) -> Result<bool, String> {
    handle
        .device
        .is_powered()
        .await
        .map_err(|_| "Could not retrieve device power state".to_string())
}

pub async fn set_device_powered(handle: &IwdHandle, is_powered: bool) -> Result<(), String> {
    handle
        .device
        .set_power(is_powered)
        .await
        .map_err(|_| "Could not set device power state".to_string())
}

pub async fn get_access_points(
    handle: &IwdHandle,
) -> Result<Vec<iwdrs::access_point::AccessPoint>, String> {
    handle
        .iwd_session
        .access_points()
        .await
        .map_err(|_| "Could not get list of access points".to_string())
}
