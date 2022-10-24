use std::collections::HashMap;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::time::Duration;
use dbus::arg::{OwnedFd, RefArg, Variant};
use dbus::blocking::{Connection, Proxy};
use dbus_udisks2::{DiskDevice, Disks, UDisks2};
use libc;

type UDisksOptions = HashMap<&'static str, Variant<Box<dyn RefArg>>>;

pub fn list_devices() -> Vec<DiskDevice>{

    let udisks = UDisks2::new().unwrap();
    let devices = Disks::new(&udisks).devices;
    let mut devices = devices.into_iter()
        .filter(|d| d.drive.connection_bus == "usb" || d.drive.connection_bus == "sdio")
        .filter(|d| d.parent.size != 0)
        .collect::<Vec<_>>();

    devices.sort_by_key(|d| d.drive.id.clone());

    devices

}


pub fn udisks_open(dbus_path: &str) -> Result<File, &str> {
    let connection = Connection::new_system().unwrap();

    let dbus_path = dbus::strings::Path::new(dbus_path).unwrap();

    let proxy =
        Proxy::new("org.freedesktop.UDisks2", &dbus_path, Duration::new(25, 0), &connection);

    let mut options = UDisksOptions::new();
    options.insert("flags", Variant(Box::new(libc::O_SYNC)));
    let res: (OwnedFd,) =
        proxy.method_call("org.freedesktop.UDisks2.Block", "OpenDevice", ("rw", options)).unwrap();

    Ok(unsafe { File::from_raw_fd(res.0.into_fd()) })
}