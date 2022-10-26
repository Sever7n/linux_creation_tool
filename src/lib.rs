use std::cmp::min;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, Write};
use std::io::Result as IoResult;

use dbus_udisks2::DiskDevice;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
use crate::linux::list_devices as list;
#[cfg(target_os= "linux")]
use self::linux::udisks_open;

#[cfg(target_os = "linux")]
mod linux;
pub mod ui;

pub const DIRECTORY: &'static str = "/home/severin/IdeaProjects/linux_creation_tool/";

///# Return
/// Outputs a hashmap containing the device name as key and the device handle as value
pub fn list_devices() -> HashMap<String, DiskDevice> {
    list()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperatingSystem {
    name: String,
    source: Source,
    pic: Source
}

impl OperatingSystem {

    pub fn new(name: String, source: Source, pic: Source) -> Self{
        Self {
            name,
            source,
            pic
        }
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
    os: Vec<OperatingSystem>
}

impl OperatingSystemList {

    pub fn get(&self, i: usize) -> Option<&OperatingSystem> {
        self.os.get(i)
    }

    pub fn empty() -> Self {
        Self {
            os: vec![]
        }
    }

    pub fn as_vec(&self) -> &Vec<OperatingSystem> {
        &self.os
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Source {
    Url(String),
    File(String)
}

pub fn load_config(path: &str) -> IoResult<OperatingSystemList> {
    let mut file = File::open(format!("{}{}", DIRECTORY, path))?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let json = serde_json::from_str(&content)?;

    Ok(json)
}

pub async fn write_os(client: Client, os: OperatingSystem, dev: DiskDevice) -> IoResult<()> {

    match os.source {
        Source::File(path) => read_write_iso(path, dev).await?,
        Source::Url(url) => download_write_iso(client, url, dev).await?
    }

    Ok(())
}

async fn download_write_iso(client: Client, url: String, dev: DiskDevice) -> IoResult<()> {

    let res = client.get(&url).send().await.unwrap();

    let total_size = res.content_length().ok_or(format!("Failed to get content length from: {}", &url)).unwrap();

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("-> "));
    pb.set_message(format!("Downloading from: {}", url));

    let mut stream = res.bytes_stream();
    let mut file = udisks_open(&dev.parent.path).unwrap();

    println!("d path: {}", dev.drive.path);

    let file_size = fs::metadata(&dev.parent.device).unwrap().len();
    file.seek(io::SeekFrom::Start(file_size)).unwrap();
    let mut downloaded = file_size;

    while let Some(item) = stream.next().await {
        let chunk = item.unwrap();
        file.write_all(&chunk).unwrap();
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(downloaded);
    }

    pb.finish();
    Ok(())

}

async fn read_write_iso(path: String, dev: DiskDevice) -> IoResult<()> {

    let mut content = fs::read(&path)?;

    let mut file = udisks_open(&dev.parent.path).unwrap();

    let file_size = fs::metadata(&dev.parent.device).unwrap().len();
    file.seek(io::SeekFrom::Start(file_size)).unwrap();

    file.write(&mut content)?;

    Ok(())

}