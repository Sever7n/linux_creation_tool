use gtk::{Application, ApplicationWindow};
use gtk::prelude::*;

const APPLICATION_ID: &str = "sever7n.linux_creation_tool";

fn main() {

    let app = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    app.connect_activate(build_ui);

    app.run();

}

fn build_ui(app: &Application) {

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Linux creation tool")
        .build();

    window.present();

}
