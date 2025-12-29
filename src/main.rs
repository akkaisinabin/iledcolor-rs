use std::error::Error;
use bluest::{Adapter, Device, L2capChannel, Uuid};
use tokio_stream::StreamExt;

use crate::image::RGBImage;

mod packet;
mod image;

const BLUETOOTH_NAME: &str = "iledcolor-21BE";
const BLUETOOTH_MAC: [u8; 6] = [0x9E, 0x19, 0x3D, 0x7C, 0x21, 0xBE];

const GENERIC_SERVICE_UUID: Uuid = Uuid::from_u128(0x00001800_0000_1000_8000_00805f9b34fb);
const DEVICE_NAME_UUID: Uuid = Uuid::from_u128(0x00002a00_0000_1000_8000_00805f9b34fb);

const WRITE_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000a950_0000_1000_8000_00805f9b34fb);
const CMD_CHARIC_UUID: Uuid = Uuid::from_u128(0x0000a951_0000_1000_8000_00805f9b34fb);
const WRITE_CHARIC_UUID: Uuid = Uuid::from_u128(0x0000a952_0000_1000_8000_00805f9b34fb);
const NOTIFY_CHARIC_UUID: Uuid = Uuid::from_u128(0x0000a953_0000_1000_8000_00805f9b34fb);

const UNKNOWN_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000ae00_0000_1000_8000_00805f9b34fb);
const UNKNOWN_CHARIC1_UUID: Uuid = Uuid::from_u128(0x0000ae01_0000_1000_8000_00805f9b34fb);
const UNKNOWN_NOTIFY_UUID: Uuid = Uuid::from_u128(0x0000ae02_0000_1000_8000_00805f9b34fb);


async fn find(name: &str) -> Result<Option<Device>, bluest::Error> {
    let adapter = Adapter::default()
        .await
        .expect("Bluetooth adapter not found");
    adapter.wait_available().await?;

    println!("starting scan");
    let mut scan = adapter.scan(&[]).await?;
    println!("scan started");
    while let Some(discovered_device) = scan.next().await {
        match discovered_device.device.name() {
            Ok(name) if name == BLUETOOTH_NAME => {
                println!("Found BLE device: {} {} {:?}", name, discovered_device.device.id(), discovered_device.adv_data.services);
                adapter.connect_device(&discovered_device.device).await?;
                return Ok(Some(discovered_device.device));
            }
            Ok(_) => {
                println!(
                    "[{}]",
                    discovered_device
                        .device
                        .name()
                        .as_deref()
                        .unwrap_or("(unknown)"),
                );
            }
            Err(e) => {
                eprintln!("Error retrieving device name: {}", e);
            }
        }
    }
    Ok(None)
}

pub fn print_bytes_hex(message: &str, bytes: &[u8]) {
    println!("{}", message);
    for byte in bytes.iter() {
        print!("{:02x} ", byte);
    }
    println!();
}

#[tokio::main]
async fn main() {
    let device = find(BLUETOOTH_NAME).await.expect("Scanning failed").expect("no device found");
    let services = device.discover_services_with_uuid(WRITE_SERVICE_UUID).await.expect("Service discovery failed");
    let write_service = services.iter().next().expect("No matching service found");
    let chars = write_service.characteristics().await.expect("Characteristic discovery failed");
    let cmd_char = chars.iter().find(|c| c.uuid() == CMD_CHARIC_UUID).expect("Characteristic 0 not found");
    let write_char = chars.iter().find(|c| c.uuid() == WRITE_CHARIC_UUID).expect("Characteristic 1 not found");
    let notify_char = chars.iter().find(|c| c.uuid() == NOTIFY_CHARIC_UUID).expect("Notify Characteristic not found");
    
    println!("Subscribing to notifications...");
    let mut updates = notify_char.notify().await.expect("Failed to subscribe to notifications");
    let mut response: Option<Result<Vec<u8>, bluest::Error>>;

    // 54 0d 0003 00 0064
    let connect_packet = packet::Packet::new(
        0x54,
        packet::Handle::Connect,
        None,
        None,
        vec![0x00],
    );
    cmd_char.write(&connect_packet.to_bytes()).await.expect("Failed to write to characteristic 1");
    print_bytes_hex("Connect Packet 1", &connect_packet.to_bytes());
    response = updates.next().await;
    println!("Received notification: {:?}", response);

    // 54 0f 0008 00 00 00 00 00 00 006b
    let connect_packet2 = packet::Packet::new(
        0x54,
        packet::Handle::FinishConnect,
        None,
        None,
        vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    );
    cmd_char.write(&connect_packet2.to_bytes()).await.expect("Failed to write to characteristic 1");
    print_bytes_hex("Connect Packet 2", &connect_packet2.to_bytes());
    response = updates.next().await;
    println!("Received notification: {:?}", response);

    let red_image = RGBImage::solid_color(48, 12, 254, 254, 0);
    let img_data = packet::ImageData::new(red_image.to_bytes());
    
    // 54 06 000d b4cb 1eaa 0000 06ee 00 00 00 03a2
    let mut begin_data: Vec<u8> = vec![];
    img_data.crc32.to_be_bytes().iter().for_each(|b| begin_data.push(*b));
    (img_data.to_bytes().len() as u32).to_be_bytes().iter().for_each(|b| begin_data.push(*b));
    begin_data.push(0x00);
    begin_data.push(0x00);
    begin_data.push(0x00);
    
    let begin_packet = packet::Packet::new(
        0x54,
        packet::Handle::StartStream,
        None,
        None,
        begin_data,
    );
    print_bytes_hex("Begin Packet", &begin_packet.to_bytes());
    cmd_char.write(&begin_packet.to_bytes()).await.expect("Failed to write to characteristic 1");
    response = updates.next().await;
    println!("Received notification: {:?}", response);

    for (index, chunk) in img_data.to_bytes().chunks(492).enumerate() {
        let packet = packet::Packet::new(
            0x54,
            packet::Handle::Continue,
            Some(index as u32),
            Some(chunk.len() as u16),
            chunk.to_vec(),
        );
        print_bytes_hex("Image Data Packet Chunk:", &packet.to_bytes());
        write_char.write(&packet.to_bytes()).await.expect("Failed to write to characteristic 1");
        response = updates.next().await;
        println!("Received notification: {:?}", response);
    }

    let end_packet = packet::Packet::new(
        0x54,
        packet::Handle::EndStream,
        None,
        None,
        vec![0x01],
    );
    print_bytes_hex("End Packet", &end_packet.to_bytes());
    write_char.write(&end_packet.to_bytes()).await.expect("Failed to write to characteristic 1");
    response = updates.next().await;
    println!("Received notification: {:?}", response);
}

