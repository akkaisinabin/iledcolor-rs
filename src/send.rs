use std::time::Duration;
use crate::{ble::{self, ILEDDev, find}, image::ILedImage, packet::{CtnData, Handle, Notification, Packet, StaData}};
use bluest::{Adapter, BluetoothUuidExt, Characteristic, Device, DeviceId, Service, Uuid, btuuid};
use image::EncodableLayout;
use log::{debug, info};
use tokio::time::sleep;
use tokio_stream::StreamExt;

pub fn print_bytes_hex(message: &str, bytes: &[u8]) { // TODO overhaul error handling and logging to use bluest errors again
    let mut output = String::new();
    output.push_str(message);
    output.push(' ');
    for byte in bytes.iter() {
        output.push_str(&format!("{:02x} ", byte));
    }
    debug!("{}", output);
}

// TODO refactor image into a pile of helper/sending functions with response and error handling.
pub async fn image(device: Device, image: ILedImage) -> Result<(), std::io::Error> {
    let dev = ILEDDev::new(device).await;
    
    debug!("Subscribing to notifications...");
    let mut updates = dev.notify_char
        .notify()
        .await
        .expect("Failed to subscribe to notifications");
    // let mut response: Result<Vec<u8>, bluest::Error>;
    
    // 54 0d 0003 00 0064
    let connect_packet = Packet::new(
        None, 
        Handle::Connect, 
        None, 
        None, 
        vec![0x00]);
    dev.cmd_char
        .write_without_response(&connect_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    print_bytes_hex("Connect Packet 1", &connect_packet.to_bytes());
    let response = Notification::from_vec_u8(
        updates
        .next()
        .await
        .expect("No response")
        .expect("Invalid response")
    );
    info!("{}", response);
    sleep(Duration::from_millis(10)).await;
    
    // 54 0f 0008 00 00 00 00 00 00 006b
    let auth_reset_packet = Packet::new(
        None,
        Handle::TestPass,
        None,
        None,
        vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    );
    dev.cmd_char
        .write_without_response(&auth_reset_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    print_bytes_hex("Connect Packet 2", &auth_reset_packet.to_bytes());
    let response = Notification::from_vec_u8(updates
        .next()
        .await
        .expect("No response")
        .expect("Invalid response")
    );
    info!("{}", response);
    // sleep(Duration::from_millis(100)).await;
    
    
    let img_data = CtnData::new(image.to_bytes());
    
    let begin_data = StaData::new(
        img_data.crc32,
        img_data.to_bytes().len() as u16 
    );
    
    let begin_packet = Packet::new(
        None,
        Handle::StartStream,
        None,
        None,
        begin_data.to_bytes()
    );
    print_bytes_hex("Begin Packet", &begin_packet.to_bytes());
    dev.cmd_char
        .write_without_response(&begin_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    let response = Notification::from_vec_u8(updates
        .next()
        .await
        .expect("No response")
        .expect("Invalid response")
    );
    info!("{}", response);
    sleep(tokio::time::Duration::from_millis(10)).await;
    
    for (index, chunk) in img_data.to_bytes().chunks(492).enumerate() {
        let packet = Packet::new(
            None,
            Handle::Continue,
            Some(index as u32),
            Some(chunk.len() as u16),
            chunk.to_vec(),
        );
        print_bytes_hex("Image Data Packet Chunk:", &packet.to_bytes());
        dev.write_char
            .write_without_response(&packet.to_bytes())
            .await
            .expect("Failed to write to characteristic 1");
        let response = Notification::from_vec_u8(updates
            .next()
            .await
            .expect("No response")
            .expect("Invalid response")
        );
        info!("{}", response);
        // sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    let end_packet = Packet::new(
        None, 
        Handle::EndStream,
        None,
        None,
        vec![0x01]
    );
    print_bytes_hex("End Packet", &end_packet.to_bytes());
    dev.write_char
        .write_without_response(&end_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    let response = Notification::from_vec_u8(updates
        .next()
        .await
        .expect("No response")
        .expect("Invalid response")
    );
    info!("{}", response);
    
    Ok(())
}
