use crate::image::ILedImage;
use clap::{ArgGroup, Parser};
use std::{error::Error, fs::File, path::PathBuf};

mod ble;
mod image;
mod packet;
mod send;

#[derive(clap::ValueEnum, Clone, Debug)]
enum ColorArg {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

impl ColorArg {
    fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            ColorArg::Red => (255, 0, 0),
            ColorArg::Green => (0, 255, 0),
            ColorArg::Blue => (0, 0, 255),
            ColorArg::Yellow => (255, 255, 0),
            ColorArg::Cyan => (0, 255, 255),
            ColorArg::Magenta => (255, 0, 255),
            ColorArg::White => (255, 255, 255),
            ColorArg::Black => (0, 0, 0),
        }
    }
}
// -s newpass [old pass] | [-b brightness] [-e enable] [-p password] [-i image | -c color] 
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    group(ArgGroup::new("input").args(["image_path", "color"]).required(true))
)]
pub struct Cli {
    #[arg(short, long)]
    pub device_name: String,
    #[arg(short, long)]
    pub image_path: Option<PathBuf>,
    #[arg(short, long)]
    color: Option<ColorArg>,
}

#[tokio::main]      // TODO rewrite main function as message queue with arg handling; stdin; sockets?
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let image: ILedImage = match cli {
        Cli {
            device_name: _,
            image_path: Some(path),
            color: _,
        } => {
            let file = File::open(path).map_err(|e| e.to_string())?;
            ILedImage::from_file(file).map_err(|e| e.to_string())?
        }
        Cli {
            device_name: _,
            image_path: None,
            color: Some(color),
        } => {
            let (r, g, b) = color.to_rgb();
            ILedImage::solid_color(48, 12, r, g, b)
        }
        _ => panic!("No input provided"),
    };

    println!("Looking for device: {}", cli.device_name);
    let device = ble::find(&cli.device_name)
        .await
        .expect("Scanning failed")
        .expect("No device found");

    println!("Sending image to device: {}", cli.device_name);
    send::image(device, image)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
