use iced::{Application, Settings};
use iced::window::Icon;
use reqwest::Client;
use linux_creation_tool::*;
use linux_creation_tool::ui::Flags;
use image::io::Reader as ImageReader;
use crate::ui::App;

fn main() {

    let client = Client::new();

    let img = ImageReader::open(format!("{}{}", DIRECTORY, "pictures/icon.png")).unwrap().decode().unwrap();
    let img = img.as_rgba8().unwrap().as_raw();

    let mut settings = Settings::default();
    settings.flags = Flags::new(client, "/etc/linux_creation_tool/");
    settings.window.size = (512, 362);
    settings.exit_on_close_request = true;
    settings.window.icon = Some(Icon::from_rgba(img.clone(), 256, 256).unwrap());

    App::run(settings).unwrap();

}