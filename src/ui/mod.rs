mod snapping_scrollbar;

use std::collections::HashMap;

use crate::ui::snapping_scrollbar::SnappingScrollable;
use crate::{
    download, list_devices, load_config, read, OperatingSystemList, Progress, Source, DIRECTORY,
};
use dbus_udisks2::DiskDevice;
use iced::Theme;
use iced::{
    alignment::Horizontal,
    executor,
    widget::{Button, Column, Image, PickList, Row, Space, Text},
    Application, Command, ContentFit, Element, Length, Padding, Subscription,
};
use iced_native::widget::ProgressBar;
use image::io::Reader as ImageReader;
use reqwest::Client;

pub struct App {
    client: Client,
    os_list: Option<OperatingSystemList>,
    disks: HashMap<String, DiskDevice>,
    disk_labels: Vec<String>,
    downloads: Option<Download>,
    reads: Option<Read>,
    last_id: usize,
    states: AppStates,
    images: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartWriting,
    SelectDevice(String),
    Scrolled(usize),
    Download(DownloadMessage),
    Read(DownloadMessage),
    None,
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    Download(usize),
    DownloadProgressed((usize, Progress)),
}

#[derive(Default, Debug)]
struct AppStates {
    error_message: Vec<String>,
    selected_region: usize,
    selected_device: Option<String>,
}

pub struct Flags {
    client: Client,
    config: &'static str,
}

impl Flags {
    pub fn new(client: Client, config: &'static str) -> Self {
        Flags { client, config }
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            client: Client::new(),
            config: "config.json",
        }
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let dev = list_devices().unwrap();
        let mut labels = Vec::new();

        dev.iter().for_each(|(l, _)| labels.push(l.clone()));

        let (os_list, images) = match load_config(flags.config) {
            Ok(c) => {
                let len = c.as_vec().len();
                let mut images = Vec::with_capacity(len);
                c.as_vec().iter().for_each(|os| {
                    images.push(match os.pic() {
                        Source::Url(_) => format!("{}{}", DIRECTORY, "pictures/missing.png"),
                        Source::File(path) => {
                            let path = format!("{}{}", DIRECTORY, &path);
                            match ImageReader::open(&path) {
                                Ok(i) => match i.decode() {
                                    Ok(_) => path,
                                    Err(_) => format!("{}{}", DIRECTORY, "pictures/missing.png"),
                                },
                                Err(_) => format!("{}{}", DIRECTORY, "pictures/missing.png"),
                            }
                        }
                    })
                });
                (Some(c), images)
            }
            Err(e) => {
                eprintln!("{e}");
                (
                    None,
                    vec![format!("{}{}", DIRECTORY, "pictures/missing.png")],
                )
            }
        };

        let app = Self {
            client: flags.client,
            os_list,
            disks: dev,
            disk_labels: labels,
            downloads: None,
            reads: None,
            last_id: 0,
            states: AppStates::default(),
            images,
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        "Linux Creation Tool".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        return match message {
            Message::StartWriting => {
                self.states.error_message = vec![];

                let os_list = match &self.os_list {
                    None => {
                        self.states
                            .error_message
                            .push("Failed to get the ISO list".into());
                        return Command::none();
                    }
                    Some(ls) => ls,
                };

                let os = match os_list.get(self.states.selected_region) {
                    None => unreachable!(),
                    Some(os) => os,
                };

                let device = match self
                    .disks
                    .get(&self.states.selected_device.clone().unwrap_or("".into()))
                {
                    None => {
                        self.states
                            .error_message
                            .push("Failed to get device".into());
                        return Command::none();
                    }
                    Some(dev) => dev,
                };

                self.last_id += 1;
                return match os.source.clone() {
                    Source::Url(url) => {
                        let mut download =
                            Download::new(self.last_id, url, device.clone(), self.client.clone());
                        download.start();

                        self.downloads = Some(download);

                        Command::none()
                    }
                    Source::File(path) => {
                        let mut read = Read::new(self.last_id, path, device.clone());
                        read.start();

                        self.reads = Some(read);

                        Command::none()
                    }
                };
            }
            Message::SelectDevice(label) => {
                self.states.selected_device = Some(label);
                Command::none()
            }
            Message::Scrolled(region) => {
                self.states.selected_region = region;

                Command::none()
            }
            Message::Download(DownloadMessage::DownloadProgressed((id, progress))) => {
                if let Some(download) = self.downloads.iter_mut().find(|download| download.id == id)
                {
                    download.progress(progress);
                }

                Command::none()
            }
            Message::Read(DownloadMessage::DownloadProgressed((id, progress))) => {
                if let Some(read) = self.reads.iter_mut().find(|read| read.id == id) {
                    read.progress(progress);
                }

                Command::none()
            }
            _ => Command::none(),
        };
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs: Vec<Subscription<Message>> = vec![];

        if let Some(download) = &self.downloads {
            subs.push(download.subscription());
        }
        if let Some(read) = &self.reads {
            subs.push(read.subscription());
        }

        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let os_list = match self.os_list.clone() {
            None => {
                // self.states
                //     .error_message
                //     .push("Failed to load OS List".into());
                OperatingSystemList::empty()
            }
            Some(ls) => ls,
        };

        let os = os_list.get(self.states.selected_region);
        let label = match os {
            Some(os) => os.name().clone(),
            _ => "".to_string(),
        };

        let text = Text::new(label)
            .horizontal_alignment(Horizontal::Center)
            .width(Length::FillPortion(100));

        let mut images = Column::new();
        for image in &self.images {
            let image = Image::new(image);
            images = images.push(image.content_fit(ContentFit::Contain));
        }

        let scrolled_image =
            SnappingScrollable::new(images, (0, os_list.as_vec().len()), (0.5, 0.5))
                .height(Length::FillPortion(50))
                .on_scroll(|region| Message::Scrolled(region.1));

        let dev_list = PickList::new(
            &self.disk_labels,
            self.states.selected_device.clone(),
            Message::SelectDevice,
        )
        .placeholder("Choose a device ...");

        let start_button =
            Button::new(Text::new("Write ISO to drive...")).on_press(Message::StartWriting);

        let mut row = Row::new().push(dev_list);

        let state = match &self.downloads {
            None => &State::Idle,
            Some(d) => d.state(),
        };

        match state {
            State::Progressing { progress } => {
                row = row.push(ProgressBar::new(0.0..=100.0, *progress));
            }
            _ => {
                let state = match &self.reads {
                    None => &State::Idle,
                    Some(d) => d.state(),
                };

                match state {
                    State::Progressing { progress } => {
                        row = row.push(ProgressBar::new(0.0..=100.0, *progress))
                    }
                    _ => row = row.push(Space::with_width(Length::Fill)).push(start_button),
                }
            }
        }

        let mut col = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(25.0))
            .push(text)
            .push(scrolled_image)
            .push(row);

        if !self.states.error_message.is_empty() {
            let mut errors = Column::new();

            for err in &self.states.error_message {
                errors = errors.push(Text::new(err));
            }

            col = col.push(errors);
        }

        col.into()
    }
}

#[derive(Debug)]
enum State {
    Idle,
    Progressing { progress: f32 },
    Finished,
    Errored,
}

#[derive(Debug)]
struct Read {
    id: usize,
    dev: DiskDevice,
    path: String,
    state: State,
}

impl Read {
    pub fn new(id: usize, path: String, dev: DiskDevice) -> Self {
        Read {
            id,
            dev,
            path,
            state: State::Idle,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle | State::Finished { .. } | State::Errored { .. } => {
                self.state = State::Progressing { progress: 0.0 };
            }
            _ => {}
        }
    }

    pub fn progress(&mut self, new_progress: Progress) {
        if let State::Progressing { progress } = &mut self.state {
            match new_progress {
                Progress::Started => *progress = 0.0,
                Progress::Advanced(percentage) => *progress = percentage,
                Progress::Finished => self.state = State::Finished,
                Progress::Errored => self.state = State::Errored,
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Progressing { .. } => read::file(self.id, &self.path, self.dev.clone())
                .map(|p| Message::Read(DownloadMessage::DownloadProgressed(p))),
            _ => Subscription::none(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

#[derive(Debug)]
struct Download {
    id: usize,
    dev: DiskDevice,
    url: String,
    state: State,
    client: Client,
}

impl Download {
    pub fn new(id: usize, url: String, dev: DiskDevice, client: Client) -> Self {
        Download {
            id,
            url,
            dev,
            state: State::Idle,
            client,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle | State::Finished { .. } | State::Errored { .. } => {
                self.state = State::Progressing { progress: 0.0 };
            }
            _ => {}
        }
    }

    pub fn progress(&mut self, new_progress: Progress) {
        if let State::Progressing { progress } = &mut self.state {
            match new_progress {
                Progress::Started => *progress = 0.0,
                Progress::Advanced(percentage) => *progress = percentage,
                Progress::Finished => self.state = State::Finished,
                Progress::Errored => self.state = State::Errored,
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Progressing { .. } => {
                download::file(self.id, &self.url, self.dev.clone(), self.client.clone())
                    .map(|p| Message::Download(DownloadMessage::DownloadProgressed(p)))
            }
            _ => Subscription::none(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}
