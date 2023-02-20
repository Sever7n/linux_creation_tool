pub mod download;
pub mod read;

use std::fs::File;
use std::io;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;

#[cfg(target_os = "linux")]
pub use crate::linux::list_devices;

#[cfg(target_os = "linux")]
mod linux;
pub mod ui;

pub const DIRECTORY: &str = "/etc/linux_creation_tool/";

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

pub fn load_config(path: &str) -> io::Result<OperatingSystemList> {
    let mut file = File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let json: OperatingSystemList = serde_json::from_str(&content)?;

    Ok(json)
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}
