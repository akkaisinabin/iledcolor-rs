use crc::{CRC_32_ISCSI, Crc};
use std::mem::size_of;

#[derive(Debug, Clone, Copy)]
pub enum Handle {
    Continue = 0x00,
    EndStream = 0x01,
    StartStream = 0x06,
    Connect = 0x0d,
    FinishConnect = 0x0f,
}

#[derive(Debug, Clone)]
pub struct Packet {
    opcode: u8, // 0x54
    handle: Handle,
    length: u16,
    sequence: Option<u32>,
    data_length: Option<u16>,
    data: Vec<u8>,
    checksum: u16, // Sum of all bytes in packet
}

impl Packet {
    pub fn new(
        opcode: u8,
        handle: Handle,
        sequence: Option<u32>,
        data_length: Option<u16>,
        data: Vec<u8>,
    ) -> Self {
        let length = data.len() as u16
            + if sequence.is_some() {
                size_of::<u32>() as u16
            } else {
                0
            }
            + if data_length.is_some() {
                size_of::<u16>() as u16
            } else {
                0
            }
            + size_of::<u16>() as u16; // Checksum
        let mut packet = Packet {
            opcode,
            handle,
            length,
            sequence,
            data_length,
            data,
            checksum: 0,
        };
        packet.checksum = packet.calculate_checksum();
        packet
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.opcode);
        bytes.push(self.handle as u8);
        bytes.extend(&self.length.to_be_bytes());
        if let Some(seq) = self.sequence {
            bytes.extend(&seq.to_be_bytes());
        }
        if let Some(data_len) = self.data_length {
            bytes.extend(&data_len.to_be_bytes());
        }
        bytes.extend(&self.data);
        bytes.extend(&self.checksum.to_be_bytes());
        bytes
    }

    fn calculate_checksum(&self) -> u16 {
        let mut sum: u16 = 0;
        sum = sum.wrapping_add(self.opcode as u16);
        sum = sum.wrapping_add(self.handle as u16);
        sum = sum.wrapping_add(self.length >> 8);
        sum = sum.wrapping_add(self.length & 0xFF);
        if let Some(seq) = self.sequence {
            sum = sum.wrapping_add((seq & 0xFF) as u16);
            sum = sum.wrapping_add(((seq >> 8) & 0xFF) as u16);
            sum = sum.wrapping_add(((seq >> 16) & 0xFF) as u16);
            sum = sum.wrapping_add(((seq >> 24) & 0xFF) as u16);
        }
        if let Some(data_len) = self.data_length {
            sum = sum.wrapping_add(data_len >> 8);
            sum = sum.wrapping_add(data_len & 0xFF);
        }
        for byte in &self.data {
            sum = sum.wrapping_add(*byte as u16);
        }
        sum
    }
}

impl std::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Packet {{ Opcode: 0x{:02X}, Handle: {:?}, Length: {}, Sequence: {:?}, Data Length: {:?}, Data: {:?}, Checksum: 0x{:04X} }}",
            self.opcode,
            self.handle,
            self.length,
            self.sequence,
            self.data_length,
            self.data,
            self.checksum
        )
    }
}

// 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00

pub struct ImageData {
    pub crc32: u32,
    start_byte: u8, // 01
    padding: [u8; 19],
    pub data: Vec<u8>,
}

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

impl ImageData {
    pub fn new(data: Vec<u8>) -> Self {
        let crc32 = CRC32.checksum(&data);
        ImageData {
            crc32,
            start_byte: 0x01,
            padding: [0; 19],
            data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.crc32.to_be_bytes());
        bytes.push(self.start_byte);
        bytes.extend(&self.padding);
        bytes.extend(&self.data);
        bytes
    }
}
