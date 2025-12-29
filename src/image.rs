use image::{
    ImageError, ImageFormat, ImageReader,
    error::{ImageFormatHint, UnsupportedError, UnsupportedErrorKind},
};
use std::{
    fs::File,
    io::{Cursor, Read},
};

pub const IMAGE_METADATA_RGB_COLOR: ImageMetadata = ImageMetadata {
    unknown1: 0x0000,
    unknown2: 0x0000,
    width: 0x0030,
    height: 0x000c,
    unknown3: 0x0000,
    unknown4: 0x0001,
    unknown5: 0x0001,
    unknown6: 0x0001,
    unknown7: 0x0032,
    unknown8: 0x0064,
    unknown9: 0x0000,
};

pub const IMAGE_METADATA_GIF: ImageMetadata = ImageMetadata {
    unknown1: 0x0000,
    unknown2: 0x0000,
    width: 0x0030,
    height: 0x000c,
    unknown3: 0x0000,
    unknown4: 0x0006,
    unknown5: 0x0001,
    unknown6: 0x0064,
    unknown7: 0x0400,
    unknown8: 0x0064,
    unknown9: 0x0000,
};

pub struct ImageMetadata {
    unknown1: u16,
    unknown2: u16,
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

fn unsupported_error(format: Option<ImageFormat>) -> ImageError {
    let format_hint = if let Some(f) = format {
        ImageFormatHint::Name(f.extensions_str().join(" "))
    } else {
        ImageFormatHint::Unknown
    };
    ImageError::Unsupported(UnsupportedError::from_format_and_kind(
        format_hint.clone(),
        UnsupportedErrorKind::Format(format_hint),
    ))
}

pub struct ILedImage {
    metadata: ImageMetadata,
    data: Vec<u8>,
}

impl ILedImage {
    pub fn new(width: u16, height: u16, image_type: ImageMetadata, data: Vec<u8>) -> Self {
        let mut metadata = image_type;
        metadata.width = width;
        metadata.height = height;
        ILedImage { metadata, data }
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

impl ILedImage {
    pub fn solid_color(width: u16, height: u16, r: u8, g: u8, b: u8) -> Self {
        let mut data = Vec::with_capacity((width as usize) * (height as usize) * 3);
        for _ in 0..(width as usize) * (height as usize) {
            data.push(r);
            data.push(g);
            data.push(b);
        }
        ILedImage::new(width, height, IMAGE_METADATA_RGB_COLOR, data)
    }

    pub fn from_file(file_path: File) -> Result<Self, image::ImageError> {
        let mut buf_reader = std::io::BufReader::new(file_path);
        let mut data = Vec::new();
        buf_reader
            .read_to_end(&mut data)
            .expect("Unable to read file data");
        let mut image_reader = ImageReader::new(Cursor::new(data.clone())).with_guessed_format()?;
        let format = image_reader.format().ok_or(unsupported_error(None))?;

        let metadata = match format {
            ImageFormat::Gif => IMAGE_METADATA_GIF,
            ImageFormat::Png
            | ImageFormat::Jpeg
            | ImageFormat::Bmp
            | ImageFormat::Tiff
            | ImageFormat::WebP => IMAGE_METADATA_RGB_COLOR,
            _ => Err(unsupported_error(Some(format)))?,
        };
        image_reader.set_format(format);
        let image = image_reader.decode()?;
        let width = image.width() as u16;
        let height = image.height() as u16;

        let iledimg = if format == ImageFormat::Gif {
            ILedImage::new(width, height, metadata, data)
        } else {
            ILedImage::new(width, height, metadata, image.to_rgb8().into_raw())
        };

        Ok(iledimg)
    }
}
