use crate::ui::App;
use iced::window::Icon;
use iced::{window::Settings as WindowSettings, Application, Settings};
use image::io::Reader as ImageReader;
use linux_creation_tool::ui::Flags;
use linux_creation_tool::*;
use reqwest::Client;

const CONFIG: &str = "/etc/linux_creation_tool/config.json";

fn main() {
    let client = Client::new();

    let img = ImageReader::open(format!("{}pictures/icon.png", DIRECTORY))
        .unwrap()
        .decode()
        .unwrap();
    let img = img.as_rgba8().unwrap().as_raw();

    let settings = Settings {
        flags: Flags::new(client, CONFIG),
        exit_on_close_request: true,
        window: WindowSettings {
            size: (512, 362),
            icon: Some(Icon::from_rgba(img.clone(), 1024, 1024).unwrap()),
            ..Default::default()
        },
        ..Default::default()
    };

    App::run(settings).unwrap();
}
