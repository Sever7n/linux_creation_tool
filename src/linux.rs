use std::collections::HashMap;
use std::ffi::OsString;
use libudev::{Context, Device, Enumerator};

pub fn list_devices() -> HashMap<OsString, OsString>{

    let context = Context::new().unwrap();

    let mut enumerator = Enumerator::new(&context).unwrap();

    let mut map = HashMap::new();

    let id_model = OsString::from("ID_MODEL");

    for device in enumerator.scan_devices().unwrap().filter(|p| filter_devices(p)) {

        let name = match device.properties().find(|p| p.name() == &id_model) {
            Some(p) => p.value().to_os_string(),
            None => device.sysname().unwrap().to_os_string(),
        };

        let path = device.devnode().unwrap().as_os_str().to_os_string();

        map.insert(name, path);

    }

    map

}

fn filter_devices(dev: &Device) -> bool {

    let empty_os_string = OsString::new();

    let dev_type = OsString::from("disk");
    let id_bus = OsString::from("ID_BUS");
    let bus_ids = [OsString::from("usb"), OsString::from("ata")];

    let is_type = dev.devtype().unwrap_or_else(|| &empty_os_string) == dev_type;
    let has_bus_id = match dev.properties().find(|p| p.name() == id_bus){
        Some(p) => bus_ids.contains(&p.value().to_os_string()),
        None => false,
    };

    is_type && has_bus_id

}