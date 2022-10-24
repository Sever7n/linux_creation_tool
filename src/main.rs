use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box};
use reqwest::Client;
use linux_creation_tool::*;

const ARCH_URL: &str = "http://mirror.chaoticum.net/arch/iso/latest/archlinux-x86_64.iso";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{

    let _reqwest_client = Client::new();

    let mut _dev = list_devices();

    let _os = load_config("");

    let os = OperatingSystem {
        os: vec![
            ("file".into(), Source::File("/some_file.iso".into())),
            ("url".into(), Source::Url("/some_url.iso".into()))
        ]
    };

    let serialize = serde_json::to_string(&os).unwrap();
    println!("{}", serialize);

    let app = Application::builder()
        .application_id("sev.linux_creation_tool")
        .build();

    app.connect_activate(build_ui);

    app.run();

    Ok(())

}

fn build_ui(app: &Application) {

    let app_box = Box::builder().build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Linux creation tool")
        .default_height(180)
        .default_width(320)
        .child(&app_box)
        .build();

    window.present();

}