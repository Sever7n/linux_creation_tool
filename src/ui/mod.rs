use std::collections::HashMap;

use dbus_udisks2::DiskDevice;
use iced::widget::{pick_list, PickList, Text};
use iced::{Application, Column, Command, ContentFit, Element, executor, Image, Length, Padding, Scrollable, scrollable};
use iced::alignment::Horizontal;
use reqwest::Client;
use crate::{DIRECTORY, list_devices, load_config, OperatingSystemList, Source, write_os};

pub struct App {
    client: Client,
    os_list: Option<OperatingSystemList>,
    disks: HashMap<String, DiskDevice>,
    disk_labels: Vec<String>,
    states: AppStates
}

#[derive(Debug, Clone)]
pub enum Message {
    StartWriting,
    SelectDevice(String),
    Scrolled(f32),
    None
}

#[derive(Default, Debug)]
struct AppStates {
    pick_list: pick_list::State<String>,
    scroll_state: scrollable::State,
    selected_os: usize,
    selected_device: Option<String>,
    error_message: Option<String>
}

pub struct Flags {
    config: &'static str
}

impl Default for Flags {

    fn default() -> Self {
        Self {
            config: "demo.json"
        }
    }

}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {

        let dev = list_devices();
        let mut labels = Vec::new();

        dev.iter().for_each(|(l, _)| labels.push(l.clone()));

        let app = Self {
            client: Client::new(),
            os_list: match load_config(flags.config) {
                Ok(c) => Some(c),
                Err(_) => None
            },
            disks: dev,
            disk_labels: labels,
            states: AppStates::default()
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        "Linux Creation Tool".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        return match message {
            Message::StartWriting => {

                let os_list = match &self.os_list {
                    None => {
                        self.states.error_message = Some("Failed to get the ISO list".into());
                        return Command::none();
                    },
                    Some(ls) => ls
                };

                let os = match os_list.get(self.states.selected_os) {
                    None => unreachable!(),
                    Some(os) => {os}
                };

                let device = match self.disks.get(&self.states.selected_device.clone().unwrap_or("".into())) {
                    None => {
                        self.states.error_message = Some("Failed to get device".into());
                        return  Command::none();
                    },
                    Some(dev) => dev
                };

                Command::perform(write_os(self.client.clone(), os.clone(), (*device).clone()), |_| {Message::None})

            },
            Message::SelectDevice(label) => {
                self.states.selected_device = Some(label);
                println!("{:?}", self.states.selected_device);
                Command::none()
            },
            Message::Scrolled(offset) => {
                let len: f32 = self.os_list.as_ref().unwrap().as_vec().len() as f32 - 1f32;

                let offset: usize = (offset * len) as usize;
                let offset: f32 = offset as f32 / len;

                self.states.scroll_state.snap_to(offset);

                Command::none()
            },
            _ => {Command::none()}
        };
    }

    fn view(&mut self) -> Element<'_, Self::Message> {

        let os_list = match self.os_list.clone() {
            None => {
                self.states.error_message = Some("Failed to load OS List".into());
                OperatingSystemList::empty()
            },
            Some(ls) => ls
        };

        let label = os_list.get(self.states.selected_os).unwrap().name();

        let text  = Text::new(label)
            .horizontal_alignment(Horizontal::Center)
            .width(Length::FillPortion(100));

        let mut images = Column::new();
        for os in os_list.as_vec() {

            let image = match os.pic.clone() {
                Source::File(f) => {
                    Image::new(format!("{}{}", DIRECTORY, f))
                },
                Source::Url(_) => {
                    Image::new(format!("{}{}", DIRECTORY, "missing.png"))
                }
            };

            images = images.push(image.content_fit(ContentFit::Contain));

        }

        let scrolled_image = Scrollable::new(&mut self.states.scroll_state)
            .width(Length::FillPortion(75))
            .height(Length::FillPortion(50))
            .on_scroll(move |offset| {
                Message::Scrolled(offset)
            })
            .push(images);

        let dev_list = PickList::new(
            &mut self.states.pick_list,
            &self.disk_labels,
            self.states.selected_device.clone(),
            |s| {
                Message::SelectDevice(s)
            }
        ).placeholder("Choose a device ...");

        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(25))
            .push(text)
            .push(scrolled_image)
            .push(dev_list)
            .into()

    }
}