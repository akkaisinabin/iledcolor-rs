use bluest::{Adapter, BluetoothUuidExt, Device, DeviceId, Uuid, btuuid,Characteristic};
use log::{debug, error, info};
use tokio_stream::StreamExt;

// pub const _BLUETOOTH_MAC: [u8; 6] = [0x9E, 0x19, 0x3D, 0x7C, 0x21, 0xBE];        leftovers..

pub const _GENERIC_SERVICE_UUID:    Uuid = Uuid::from_u128(0x00001800_0000_1000_8000_00805f9b34fb);
pub const _DEVICE_NAME_UUID:        Uuid = Uuid::from_u128(0x00002a00_0000_1000_8000_00805f9b34fb);

pub const WRITE_SERVICE_UUID:       Uuid = Uuid::from_u128(0x0000a950_0000_1000_8000_00805f9b34fb);
pub const CMD_CHARIC_UUID:          Uuid = Uuid::from_u128(0x0000a951_0000_1000_8000_00805f9b34fb);
pub const WRITE_CHARIC_UUID:        Uuid = Uuid::from_u128(0x0000a952_0000_1000_8000_00805f9b34fb);
pub const NOTIFY_CHARIC_UUID:       Uuid = Uuid::from_u128(0x0000a953_0000_1000_8000_00805f9b34fb);

pub const _UNKNOWN_SERVICE_UUID:    Uuid = Uuid::from_u128(0x0000ae00_0000_1000_8000_00805f9b34fb);
pub const _UNKNOWN_CHARIC1_UUID:    Uuid = Uuid::from_u128(0x0000ae01_0000_1000_8000_00805f9b34fb);
pub const _UNKNOWN_NOTIFY_UUID:     Uuid = Uuid::from_u128(0x0000ae02_0000_1000_8000_00805f9b34fb);


#[derive(Debug, Clone)]
pub struct ILEDDev {
    pub cmd_char: Characteristic,
    pub write_char: Characteristic,
    pub notify_char: Characteristic,
    pub name_char: Characteristic,
}

impl ILEDDev {
    pub async fn new(device: Device) ->  Self {
        let services = device
            .services()
            .await
            .expect("Service discovery failed");
        let write_service = services
            .iter()
            .find(|s|s.uuid() == WRITE_SERVICE_UUID)
            .expect("No matching service found");
        let chars = write_service
            .characteristics()
            .await
            .expect("Characteristic discovery failed");
        let gen_service = services
            .iter()
            .find(|s|s.uuid() == _GENERIC_SERVICE_UUID)
            .expect("Generic service not found");
        let gen_chars = gen_service
            .characteristics()
            .await
            .expect("generic characteristics not found");
        ILEDDev {
            cmd_char: chars
                .iter()
                .find(|c| c.uuid() == CMD_CHARIC_UUID)
                .expect("Characteristic 0 not found").clone(),
            write_char: chars
                .iter()
                .find(|c| c.uuid() == WRITE_CHARIC_UUID)
                .expect("Characteristic 1 not found").clone(),
            notify_char: chars
                .iter()
                .find(|c| c.uuid() == NOTIFY_CHARIC_UUID)
                .expect("Notify Characteristic not found").clone(),
            name_char: gen_chars
                .iter()
                .find(|c|c.uuid() == _DEVICE_NAME_UUID)
                .expect("name characteristic not found").clone(),
        }
        
    }
}

pub async fn find(name: &str) -> Result<Option<Device>, bluest::Error> {
    let adapter = Adapter::default()
        .await
        .expect("Bluetooth adapter not found");
    adapter.wait_available().await?;

    debug!("Check for connected devices");
    let connected_devices = adapter.connected_devices().await?;
    for device in connected_devices {
        if let Ok(dev_name) = device.name()
            && dev_name == name
        {
            info!("Found connected BLE device: {} {}", dev_name, device.id());
            return Ok(Some(device));
        }
    }

    info!("Could not find connected device, starting scan...");
    let mut scan = adapter.scan(&[]).await?;
    while let Some(discovered_device) = scan.next().await {
        match discovered_device.device.name() {
            Ok(dev_name) if dev_name == name => {
                info!("Found {}", dev_name);
                debug!(
                    "Found BLE device: {} {} {:?}",
                    dev_name,
                    discovered_device.device.id(),
                    discovered_device.adv_data.services
                );
                adapter.connect_device(&discovered_device.device).await?;
                return Ok(Some(discovered_device.device));
            }
            Ok(_) => {
                debug!(
                    "[{}]",
                    discovered_device
                        .device
                        .name()
                        .as_deref()
                        .unwrap_or("(unknown)"),
                );
            }
            Err(e) => {
                error!("Error retrieving device name: {}", e);
            }
        }
    }
    Ok(None)
}
