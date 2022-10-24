use std::cmp::min;
use std::fs;
use std::io::{self, Read, Seek, Write};
use std::io::Result as IoResult;
use dbus_udisks2::DiskDevice;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::linux::list_devices as list;
#[cfg(target_os= "linux")]
use self::linux::udisks_open;

///# Return
/// Outputs a hashmap containing the device name as key and the device handle as value
pub fn list_devices() -> Vec<DiskDevice> {
    list()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OperatingSystem {
    pub os: Vec<(String, Source)>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Source {
    Url(String),
    File(String)
}

pub fn load_config(path: &str) -> IoResult<OperatingSystem> {
    let mut file = fs::File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let json = serde_json::from_str(&content)?;

    Ok(json)
}

pub async fn download_write_os(client: &Client, url: &str, dev: DiskDevice) -> Result<(), String> {

    println!("Downloading ...");

    let res = client.get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;

    let total_size = res.content_length().ok_or(format!("Failed to get content length from: {}", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("-> "));
    //pb.set_message(&format!("Downloading {}", url));

    let mut stream = res.bytes_stream();

    let mut file = udisks_open(&dev.parent.path).unwrap();

    println!("d path: {}", dev.drive.path);

    let file_size = fs::metadata(dev.parent.device).unwrap().len();
    file.seek(io::SeekFrom::Start(file_size)).unwrap();
    let mut downloaded = file_size;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(downloaded);
    }

    pb.finish();
    Ok(())

}