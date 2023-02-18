use dbus::arg::{OwnedFd, RefArg, Variant};
use dbus::blocking::{Connection, Proxy};
use dbus::Error;
use dbus_udisks2::{DiskDevice, Disks, UDisks2};
use std::collections::HashMap;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::time::Duration;

type UDisksOptions = HashMap<String, Variant<Box<dyn RefArg>>>;

pub fn list_devices() -> Result<HashMap<String, DiskDevice>, Error> {
    let udisks = UDisks2::new()?;
    let devices = Disks::new(&udisks).devices;

    let mut map = HashMap::new();

    devices
        .into_iter()
        .filter(|d| d.drive.connection_bus == "usb" || d.drive.connection_bus == "sdio")
        .filter(|d| d.parent.size != 0)
        .for_each(|d| {
            let label = match d.drive.vendor.is_empty() {
                true => d.drive.model.to_string(),
                false => format!("{} {}", d.drive.vendor, d.drive.model),
            };

            map.insert(label, d);
        });

    Ok(map)
}

pub fn udisks_open(dbus_path: &str) -> Result<File, Error> {
    let connection = Connection::new_system()?;

    let dbus_path = match dbus::strings::Path::new(dbus_path) {
        Ok(p) => p,
        Err(e) => return Err(Error::new_failed(&e)),
    };

    let proxy = Proxy::new(
        "org.freedesktop.UDisks2",
        &dbus_path,
        Duration::new(25, 0),
        &connection,
    );

    let mut options = UDisksOptions::new();
    options.insert("flags".into(), Variant(Box::new(libc::O_SYNC)));
    let res: (OwnedFd,) = proxy.method_call(
        "org.freedesktop.UDisks2.Block",
        "OpenDevice",
        ("rw", options),
    )?;

    Ok(unsafe { File::from_raw_fd(res.0.into_fd()) })
}
