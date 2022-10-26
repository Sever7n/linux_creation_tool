use std::error::Error;
use iced::{Application, Settings};
use linux_creation_tool::*;
use crate::ui::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{

    App::run(Settings::default()).unwrap();

    Ok(())

}