use crc::{CRC_32_ISCSI, Crc};
use std::{ffi::os_str::Display, mem::size_of};
use strum_macros::{self, FromRepr, Display};
#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr, Display)]
pub enum Handle {           // TODO find remaining handles, maybe with fuzzing?
    Continue = 0x00,
    EndStream = 0x01,
    StartStream = 0x06,
    Brightness = 0x09,
    LedEnable = 0x0A,
    Connect = 0x0D,
    SetPass = 0x0E,
    TestPass = 0x0F,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Data {         // TODO integrate this enum with Packet, write a generic Impl for it.
    Gen(Vec<u8>),
    Ctn(CtnData),
    End(u8),
    Sta(StaData),
    Con(u8),
    POp{opcode:u8, old_Pass:[u8;6], new_Pass:[u8;6]},
    Pas([u8;6]),
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
        opcode: Option<u8>,
        handle: Handle,
        sequence: Option<u32>, // TODO refactor sequence and data_length 
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
            opcode: opcode.unwrap_or(0x54),
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
            "Packet {{ Opcode: 0x{:02X}, Handle: {}, Length: {}, Sequence: {:?}, Data Length: {:?}, Data: {:?}, Checksum: 0x{:04X} }}",
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

#[repr(u8)]
#[derive(Debug, Clone, FromRepr, Display)]
pub enum GenRes {
    Success = 1,
    Fail = 2,
    Unknown,
}
#[repr(u8)]
#[derive(Debug, Clone, FromRepr, Display)]
pub enum TestPassRes {
    Correct = 1,
    Incorrect = 2,
    NoPass = 3,
    Unknown,
}

#[derive(Debug, Clone, Display)]
pub enum NotificationType {           // TODO consider wether this enum layer is needed for state machine, maybe replace named varients with the few types involved
    #[strum(to_string = "chunk number {chunk:?}")]
    Continue{chunk: u8},
    #[strum(transparent)]
    EndStream(GenRes),
    #[strum(to_string = "chunks expected: {chunks:?}+1")]
    StartStream{chunks: u8}, //TODO: probably also has success value in another field, to be tested
    #[strum(transparent)]
    Brightness(GenRes),
    #[strum(transparent)]
    LedEnable(GenRes),
    #[strum(to_string = "{0:?}")]
    Connect([u8;2]),                 //TODO: still don't really know what connect does, we send 0x00, receive 0x0000, always.
    #[strum(transparent)]
    SetPass(GenRes),
    #[strum(transparent)]
    TestPass(TestPassRes),
    #[strum(to_string = "Unkown: {0:?}")]
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Notification {
    opcode: u8, // 0x54
    handle: Handle,
    length: u16,
    data: NotificationType,
    checksum: u16, // Sum of all bytes in packet
}
impl Notification {
    // TODO add comparisons and errors for opcode; handle; length; checksum.
    // TODO refactor return into a Result<> so upstream can choose how to handle things, not all errors are critical
    pub fn from_vec_u8(response: Vec<u8>) -> Self {
        let handle = Handle::from_repr(response[1]).unwrap();
        Notification { 
            opcode: response[0],
            handle,
            length: u16::from_be_bytes(response[2..4].try_into().unwrap()),
            data: match handle {
                Handle::Continue => NotificationType::Continue{chunk: response[7]},
                Handle::EndStream => NotificationType::EndStream(GenRes::from_repr(response[4]).unwrap()),
                Handle::StartStream => NotificationType::StartStream{chunks: response[4]},
                Handle::Brightness => NotificationType::Brightness(GenRes::from_repr(response[4]).unwrap()),
                Handle::LedEnable => NotificationType::LedEnable(GenRes::from_repr(response[4]).unwrap()),
                Handle::Connect => NotificationType::Connect(response[4..6].try_into().unwrap()),
                Handle::TestPass => NotificationType::TestPass(TestPassRes::from_repr(response[4]).unwrap()),
                Handle::SetPass => NotificationType::SetPass(GenRes::from_repr(response[4]).unwrap()),
                _ => NotificationType::Unknown(response[4..response.len()-2].into()),
            },
            checksum: u16::from_be_bytes(response[response.len()-2 ..].try_into().unwrap()),
        }
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
            f,
            "Notification {{ Opcode: 0x{:02X}, Handle: {}, Length: {}, Data: {}, Checksum: 0x{:04X} }}",
            self.opcode,
            self.handle,
            self.length,
            self.data,
            self.checksum
        )
    }
}

// 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00

#[derive(Debug, Clone)]
pub struct CtnData {
    pub crc32: u32,
    start_byte: u8, // 01
    padding: [u8; 19],
    pub data: Vec<u8>,
}

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

impl CtnData {
    pub fn new(data: Vec<u8>) -> Self {
        let crc32 = CRC32.checksum(&data);
        CtnData {
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

#[derive(Debug, Clone)]
pub struct StaData {
    pub crc32: u32,
    unk1: [u8;2], // 0000
    len: u16,
    unk2: [u8; 3], // 000000
}

impl StaData {
    pub fn new(crc32: u32, len: u16) -> Self {
        StaData {
            crc32,
            unk1: [0; 2],
            len,
            unk2: [0; 3],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.crc32.to_be_bytes());
        bytes.extend(self.unk1);
        bytes.extend(&self.len.to_be_bytes());
        bytes.extend(&self.unk2);
        bytes
    }
}
