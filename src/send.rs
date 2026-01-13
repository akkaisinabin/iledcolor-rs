use std::time::Duration;

use crate::{ble, image::ILedImage, packet};
use bluest::Device;
use log::debug;
use tokio::time::sleep;
use tokio_stream::StreamExt;

pub fn print_bytes_hex(message: &str, bytes: &[u8]) {
    let mut output = String::new();
    output.push_str(message);
    output.push(' ');
    for byte in bytes.iter() {
        output.push_str(&format!("{:02x} ", byte));
    }
    debug!("{}", output);
}

pub async fn image(device: Device, image: ILedImage) -> Result<(), std::io::Error> {
    let services = device
        .discover_services_with_uuid(ble::WRITE_SERVICE_UUID)
        .await
        .expect("Service discovery failed");
    let write_service = services.first().expect("No matching service found");
    let chars = write_service
        .characteristics()
        .await
        .expect("Characteristic discovery failed");
    let cmd_char = chars
        .iter()
        .find(|c| c.uuid() == ble::CMD_CHARIC_UUID)
        .expect("Characteristic 0 not found");
    let write_char = chars
        .iter()
        .find(|c| c.uuid() == ble::WRITE_CHARIC_UUID)
        .expect("Characteristic 1 not found");
    let notify_char = chars
        .iter()
        .find(|c| c.uuid() == ble::NOTIFY_CHARIC_UUID)
        .expect("Notify Characteristic not found");

    let img_data = packet::ImageData::new(image.to_bytes());

    debug!("Subscribing to notifications...");
    let mut updates = notify_char
        .notify()
        .await
        .expect("Failed to subscribe to notifications");
    let mut response: Result<Vec<u8>, bluest::Error>;

    // 54 0d 0003 00 0064
    let connect_packet = packet::Packet::new(
        0x54, 
        packet::Handle::Connect, 
        None, 
        None, 
        vec![0x00]);
    cmd_char
        .write_without_response(&connect_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    print_bytes_hex("Connect Packet 1", &connect_packet.to_bytes());
    response = updates.next().await.expect("No response");
    print_bytes_hex("Response:", response.expect("Invalid response").as_slice());
    sleep(Duration::from_millis(10)).await;

    // 54 0f 0008 00 00 00 00 00 00 006b
    let connect_packet2 = packet::Packet::new(
        0x54,
        packet::Handle::FinishConnect,
        None,
        None,
        vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    );
    cmd_char
        .write_without_response(&connect_packet2.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    print_bytes_hex("Connect Packet 2", &connect_packet2.to_bytes());
    response = updates.next().await.expect("No response");
    print_bytes_hex("Response:", response.expect("Invalid response").as_slice());
    sleep(Duration::from_millis(10)).await;

    // 54 06 000d b4cb 1eaa 0000 06ee 00 00 00 03a2
    let mut begin_data: Vec<u8> = vec![];
    img_data
        .crc32
        .to_be_bytes()
        .iter()
        .for_each(|b| begin_data.push(*b));
    (img_data.to_bytes().len() as u32)
        .to_be_bytes()
        .iter()
        .for_each(|b| begin_data.push(*b));
    begin_data.push(0x00);
    begin_data.push(0x00);
    begin_data.push(0x00);

    let begin_packet =
        packet::Packet::new(0x54, packet::Handle::StartStream, None, None, begin_data);
    print_bytes_hex("Begin Packet", &begin_packet.to_bytes());
    cmd_char
        .write_without_response(&begin_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    response = updates.next().await.expect("No response");
    print_bytes_hex("Response:", response.expect("Invalid response").as_slice());
    sleep(tokio::time::Duration::from_millis(100)).await;

    for (index, chunk) in img_data.to_bytes().chunks(492).enumerate() {
        let packet = packet::Packet::new(
            0x54,
            packet::Handle::Continue,
            Some(index as u32),
            Some(chunk.len() as u16),
            chunk.to_vec(),
        );
        print_bytes_hex("Image Data Packet Chunk:", &packet.to_bytes());
        write_char
            .write_without_response(&packet.to_bytes())
            .await
            .expect("Failed to write to characteristic 1");
        response = updates.next().await.expect("No response");
        sleep(tokio::time::Duration::from_millis(10)).await;
        print_bytes_hex("Response:", response.expect("Invalid response").as_slice());
    }

    let end_packet = packet::Packet::new(0x54, packet::Handle::EndStream, None, None, vec![0x01]);
    print_bytes_hex("End Packet", &end_packet.to_bytes());
    write_char
        .write_without_response(&end_packet.to_bytes())
        .await
        .expect("Failed to write to characteristic 1");
    response = updates.next().await.expect("No response");
    print_bytes_hex("Response:", response.expect("Invalid response").as_slice());
    Ok(())
}
