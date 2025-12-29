use clap::{ArgGroup, Parser, command};

use crate::image::Image;

mod packet;
mod image;
mod ble;
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
    pub image_path: Option<String>,
    #[arg(short, long)]
    color: Option<ColorArg>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let cli = Cli::parse();
    let device = ble::find(&cli.device_name).await.expect("Scanning failed").expect("no device found");
    
    let image: Image = match cli {
        Cli {device_name: _ , image_path: Some(path), color: _ } => 
            Image::from_file(std::fs::File::open(path)?).expect("Failed to load image"),
        Cli {device_name: _, image_path: None, color: Some(color) } => {
            let (r, g, b) = color.to_rgb();
            Image::solid_color(48, 12, r, g, b)
        },
        _ => panic!("No input provided"),
    };

    send::image(device, image).await
}

