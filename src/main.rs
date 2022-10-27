use std::error::Error;
use iced::{Application, Settings};
use iced::window::Icon;
use reqwest::Client;
use linux_creation_tool::*;
use linux_creation_tool::ui::Flags;
use image::io::Reader as ImageReader;
use crate::ui::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{

    let client = Client::new();

    let img = ImageReader::open(format!("{}{}", DIRECTORY, "icon.png"))?.decode()?;

    let img = img.as_rgba8().unwrap().as_raw();

    let mut settings = Settings::default();
    settings.flags = Flags::new(client, "demo.json");
    settings.window.size = (512, 362);
    settings.exit_on_close_request = true;
    settings.window.icon = Some(Icon::from_rgba(img.clone(), 32, 32)?);

    App::run(settings).unwrap();

    Ok(())

}