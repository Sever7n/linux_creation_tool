use std::cmp::min;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::Result as IoResult;
use std::io::{self, Read, Seek, Write};

use dbus_udisks2::DiskDevice;

use iced_native::subscription;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
use self::linux::udisks_open;
#[cfg(target_os = "linux")]
use crate::linux::list_devices as list;

#[cfg(target_os = "linux")]
mod linux;
pub mod ui;

pub const DIRECTORY: &str = "/etc/linux_creation_tool/";

///# Return
/// Outputs a hashmap containing the device name as key and the device handle as value
pub fn list_devices() -> HashMap<String, DiskDevice> {
    list()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperatingSystem {
    name: String,
    source: Source,
    pic: Source,
}

impl OperatingSystem {
    pub fn new(name: String, source: Source, pic: Source) -> Self {
        Self { name, source, pic }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn pic(&self) -> &Source {
        &self.pic
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperatingSystemList {
    os: Vec<OperatingSystem>,
}

impl OperatingSystemList {
    pub fn get(&self, i: usize) -> Option<&OperatingSystem> {
        self.os.get(i)
    }

    pub fn empty() -> Self {
        Self { os: vec![] }
    }

    pub fn as_vec(&self) -> &Vec<OperatingSystem> {
        &self.os
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Source {
    Url(String),
    File(String),
}

pub fn load_config(path: &str) -> IoResult<OperatingSystemList> {
    let mut file = File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let json: OperatingSystemList = serde_json::from_str(&content)?;

    Ok(json)
}

pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    url: T,
    dev: DiskDevice,
    client: Client,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(
        id,
        State::Ready(url.to_string(), Box::new(dev), client),
        move |state| download(id, state),
    )
}

#[derive(Debug, Hash, Clone)]
pub struct Download<I> {
    id: I,
    url: String,
}

async fn download<I: Copy>(id: I, state: State) -> (Option<(I, Progress)>, State) {
    match state {
        State::Ready(url, dev, client) => {
            let response = client.get(&url).send().await;

            match response {
                Ok(response) => {
                    if let Some(total) = response.content_length() {
                        let mut file = match udisks_open(&dev.parent.path) {
                            Ok(f) => f,
                            Err(_) => return (Some((id, Progress::Errored)), State::Finished),
                        };

                        let file_size = fs::metadata(&dev.parent.device).unwrap().len();
                        file.seek(io::SeekFrom::Start(file_size)).unwrap();

                        (
                            Some((id, Progress::Started)),
                            State::Downloading {
                                response,
                                file,
                                total,
                                downloaded: 0,
                            },
                        )
                    } else {
                        (Some((id, Progress::Errored)), State::Finished)
                    }
                }
                Err(_) => (Some((id, Progress::Errored)), State::Finished),
            }
        }
        State::Downloading {
            mut response,
            mut file,
            total,
            downloaded,
        } => match response.chunk().await {
            Ok(None) => (Some((id, Progress::Finished)), State::Finished),
            Ok(Some(chunk)) => {
                if file.write_all(&chunk).is_ok() {
                    let new = min(downloaded + (chunk.len() as u64), total);

                    let percentage = (new as f32 / total as f32) * 100.0;

                    (
                        Some((id, Progress::Advanced(percentage))),
                        State::Downloading {
                            response,
                            total,
                            downloaded: new,
                            file,
                        },
                    )
                } else {
                    (Some((id, Progress::Errored)), State::Finished)
                }
            }
            Err(_) => (Some((id, Progress::Errored)), State::Finished),
        },
        State::Finished => iced::futures::future::pending().await,
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}

pub enum State {
    Ready(String, Box<DiskDevice>, Client),
    Downloading {
        response: Response,
        file: File,
        total: u64,
        downloaded: u64,
    },
    Finished,
}

pub async fn read_write_iso(path: String, dev: DiskDevice) -> IoResult<()> {
    let content = fs::read(path)?;

    let mut file = udisks_open(&dev.parent.path).unwrap();

    let file_size = fs::metadata(&dev.parent.device).unwrap().len();
    file.seek(io::SeekFrom::Start(file_size)).unwrap();

    file.write_all(&content)?;

    Ok(())
}
