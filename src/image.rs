struct ImageMetadata {
    Unknown1 : u16, // 00 00
    Unknown2 : u16, // 00 00
    width: u16, // 0030
    height: u16, // 000c
    Unknown3: u16, // 00 00
    Unknown4: u16, // 00 01
    Unknown5: u16, // 00 01
    Unknown6: u16, // 00 01
    Unknown7: u16, // 32 00  64 04
    Unknown8: u16, // 64 00
    Unknown9: u16, // 00 00
}

pub struct RGBImage {
    metadata: ImageMetadata,
    data: Vec<u8>,
}

impl RGBImage {
    pub fn new(width: u16, height: u16, data: Vec<u8>) -> Self {
        if data.len() != (width as usize) * (height as usize) * 3 {
            panic!("Data length does not match width and height");
        }
        let metadata = ImageMetadata {
            Unknown1: 0,
            Unknown2: 0,
            width,
            height,
            Unknown3: 0,
            Unknown4: 1,
            Unknown5: 1,
            Unknown6: 1,
            Unknown7: 0x3200,
            Unknown8: 0x6400,
            Unknown9: 0,
        };
        RGBImage { metadata, data }
    }

    pub fn solid_color(width: u16, height: u16, r: u8, g: u8, b: u8) -> Self {
        let mut data = Vec::with_capacity((width as usize) * (height as usize) * 3);
        for _ in 0..(width as usize) * (height as usize) {
            data.push(r);
            data.push(g);
            data.push(b);
        }
        RGBImage::new(width, height, data)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.metadata.Unknown1.to_be_bytes());
        bytes.extend(&self.metadata.Unknown2.to_be_bytes());
        bytes.extend(&self.metadata.width.to_be_bytes());
        bytes.extend(&self.metadata.height.to_be_bytes());
        bytes.extend(&self.metadata.Unknown3.to_be_bytes());
        bytes.extend(&self.metadata.Unknown4.to_be_bytes());
        bytes.extend(&self.metadata.Unknown5.to_be_bytes());
        bytes.extend(&self.metadata.Unknown6.to_be_bytes());
        bytes.extend(&self.metadata.Unknown7.to_be_bytes());
        bytes.extend(&self.metadata.Unknown8.to_be_bytes());
        bytes.extend(&self.metadata.Unknown9.to_be_bytes());
        bytes.extend(&self.data);
        bytes
    }
}


