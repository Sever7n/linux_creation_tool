use std::collections::HashMap;
use std::cmp::min;
use std::ffi::OsString;
use std::fs;
use std::io::{Seek, Write};

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use futures_util::StreamExt;
use polkit::ffi::polkit_action_description_get_action_id;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::linux::list_devices as list;

///# Return
/// Outputs a hashmap containing the device name as key and the device handle as value
pub fn list_devices() -> HashMap<OsString, OsString> {
    list()
}

/*
pub struct OperatingSystem {
    name: String,
    url: String,
}
*/

pub async fn download_write_os(client: &Client, url: &str, dev: &str) -> Result<(), String> {

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



    file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(dev)
        .unwrap();

    let file_size = fs::metadata(dev).unwrap().len();
    file.seek(std::io::SeekFrom::Start(file_size)).unwrap();
    let mut downloaded = file_size;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(downloaded);
    }

    Ok(())

}