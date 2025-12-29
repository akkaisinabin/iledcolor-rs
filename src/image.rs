use std::{fs::File, io::{Cursor, Read}};
use image::ImageReader;

pub const IMAGE_METADATA_RGB_COLOR: [u8; 22] = [
    0x00, 0x00, // unknown1
    0x00, 0x00, // unknown2
    0x00, 0x30, // width
    0x00, 0x0c, // height
    0x00, 0x00, // unknown3
    0x00, 0x01, // unknown4
    0x00, 0x01, // unknown5
    0x00, 0x01, // unknown6
    0x32, 0x00, // unknown7
    0x64, 0x00, // unknown8
    0x00, 0x00, // unknown9
];

pub const IMAGE_METADATA_GIF: [u8; 22] = [
    0x00, 0x00, // unknown1
    0x00, 0x00, // unknown2
    0x00, 0x30, // width
    0x00, 0x0c, // height
    0x00, 0x00, // unknown3
    0x00, 0x06, // unknown4
    0x00, 0x01, // unknown5
    0x00, 0x64, // unknown6
    0x04, 0x00, // unknown7
    0x64, 0x00, // unknown8
    0x00, 0x00, // unknown9
];

pub struct ImageMetadata {
    unknown1 : u16,
    unknown2 : u16,
    width: u16,
    height: u16,
    unknown3: u16,
    unknown4: u16,
    unknown5: u16,
    unknown6: u16,
    unknown7: u16,
    unknown8: u16,
    unknown9: u16,
}

impl ImageMetadata {
    fn from_bytes(bytes: &[u8]) -> Option<ImageMetadata> {
        let mut b = bytes.iter();
        Some(ImageMetadata {
            unknown1: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown2: u16::from_be_bytes([*b.next()?, *b.next()?]),
            width: u16::from_be_bytes([*b.next()?, *b.next()?]),
            height: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown3: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown4: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown5: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown6: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown7: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown8: u16::from_be_bytes([*b.next()?, *b.next()?]),
            unknown9: u16::from_be_bytes([*b.next()?, *b.next()?]),
        })
    }
}

pub struct Image {
    metadata: ImageMetadata,
    data: Vec<u8>,
}

impl Image {
    pub fn new(width: u16, height: u16, image_type: ImageMetadata, data: Vec<u8>) -> Self {
        let mut metadata = image_type;
        metadata.width = width;
        metadata.height = height;
        Image { metadata, data }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.metadata.unknown1.to_be_bytes());
        bytes.extend(&self.metadata.unknown2.to_be_bytes());
        bytes.extend(&self.metadata.width.to_be_bytes());
        bytes.extend(&self.metadata.height.to_be_bytes());
        bytes.extend(&self.metadata.unknown3.to_be_bytes());
        bytes.extend(&self.metadata.unknown4.to_be_bytes());
        bytes.extend(&self.metadata.unknown5.to_be_bytes());
        bytes.extend(&self.metadata.unknown6.to_be_bytes());
        bytes.extend(&self.metadata.unknown7.to_be_bytes());
        bytes.extend(&self.metadata.unknown8.to_be_bytes());
        bytes.extend(&self.metadata.unknown9.to_be_bytes());
        bytes.extend(&self.data);
        bytes
    }
}

impl Image {
    pub fn solid_color(width: u16, height: u16, r: u8, g: u8, b: u8) -> Self {
        let mut data = Vec::with_capacity((width as usize) * (height as usize) * 3);
        for _ in 0..(width as usize) * (height as usize) {
            data.push(r);
            data.push(g);
            data.push(b);
        }
        Image::new(width, height, ImageMetadata::from_bytes(&IMAGE_METADATA_RGB_COLOR).expect("Unable to cast bytes to ImageMetadata"), data)
    }

    pub fn from_file(file_path: File) -> Result<Self, image::ImageError> {
        let mut buf_reader = std::io::BufReader::new(file_path);
        let mut data = Vec::new();
        buf_reader.read_to_end(&mut data).expect("Unable to read file data");
        let image = ImageReader::new(Cursor::new(data.clone())).with_guessed_format()?.decode()?;
        let width = image.width() as usize;
        let height: usize = image.height() as usize;
        Ok(Image::new(width as u16, height as u16, ImageMetadata::from_bytes(&IMAGE_METADATA_GIF).expect("Unable to cast bytes to ImageMetadata"), data))
    }
}
