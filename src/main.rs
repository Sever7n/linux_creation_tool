use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box};
use reqwest::Client;
use linux_creation_tool::list_devices;
use linux_creation_tool::download_write_os;

const ARCH_URL: &str = "http://mirror.chaoticum.net/arch/iso/latest/archlinux-x86_64.iso";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{

    let dev = list_devices();
    
    println!("{:?}", dev);

    match download_write_os(&Client::new(), ARCH_URL, "/dev/sdb").await {
        Ok(()) => {},
        Err(e) => eprintln!("{}", e)
    };

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