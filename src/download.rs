use dbus_udisks2::DiskDevice;
use iced::subscription;
use reqwest::Client;
use reqwest::Response;

use std::cmp::min;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::io::Seek;
use std::io::Write;

#[cfg(target_os = "linux")]
use crate::linux::udisks_open;

use crate::Progress;

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

                        let file_size = match fs::metadata(&dev.parent.device) {
                            Ok(x) => x.len(),
                            Err(_) => return (Some((id, Progress::Errored)), State::Finished),
                        };

                        if file.seek(io::SeekFrom::Start(file_size)).is_err() {
                            return (Some((id, Progress::Errored)), State::Finished);
                        };

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
