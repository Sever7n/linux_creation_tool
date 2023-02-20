use std::{
    cmp::min,
    fs::{self, File},
    hash::Hash,
    io::{self, BufReader, Read, Seek, Write},
};

use dbus_udisks2::DiskDevice;
use iced::subscription;

#[cfg(target_os = "linux")]
use crate::linux::udisks_open;
use crate::Progress;

pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    path: T,
    dev: DiskDevice,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(
        id,
        State::Ready(path.to_string(), Box::new(dev)),
        move |state| read(id, state),
    )
}

async fn read<I: Copy>(id: I, state: State) -> (Option<(I, Progress)>, State) {
    match state {
        State::Ready(path, dev) => {
            let content = match File::open(path) {
                Ok(f) => f,
                Err(_) => return (Some((id, Progress::Errored)), State::Finished),
            };

            let total = match content.metadata() {
                Ok(m) => m.len(),
                Err(_) => return (Some((id, Progress::Errored)), State::Finished),
            };

            let reader = BufReader::new(content);

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
            }

            (
                Some((id, Progress::Started)),
                State::Reading {
                    reader,
                    file,
                    total,
                    read: 0,
                },
            )
        }
        State::Reading {
            mut reader,
            mut file,
            total,
            read,
        } => {
            let mut buffer = [0; 1048576];
            let size = match reader.read(&mut buffer) {
                Ok(size) => size,
                Err(_) => return (Some((id, Progress::Errored)), State::Finished),
            };

            if size == 0 {
                return (Some((id, Progress::Finished)), State::Finished);
            }

            if file.write_all(&buffer).is_err() {
                return (Some((id, Progress::Errored)), State::Finished);
            }

            let new = min(read + buffer.len() as u64, total);
            let percentage = (new as f32 / total as f32) * 100.0;

            (
                Some((id, Progress::Advanced(percentage))),
                State::Reading {
                    reader,
                    file,
                    total,
                    read: new,
                },
            )
        }
        State::Finished => iced::futures::future::pending().await,
    }
}

pub enum State {
    Ready(String, Box<DiskDevice>),
    Reading {
        reader: BufReader<File>,
        file: File,
        total: u64,
        read: u64,
    },
    Finished,
}
