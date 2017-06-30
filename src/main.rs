extern crate libusb;
extern crate hex_utils;
extern crate libxdo;
extern crate yaml_rust;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate rand;

#[macro_use]
mod macros;
mod device_manager;
mod device_input;
mod device_mapping;
mod input;
mod map_input;
mod profile_definition;


use std::{thread, time};

use device_mapping::DeviceMaps;
use device_manager::DeviceManager;

use profile_definition::Profiles;


fn main() {
    let mappings = DeviceMaps::new("devices.yaml").unwrap();
    let device_inputs = mappings.get_inputs();
    let profiles = Profiles::new("profile.yaml", device_inputs).unwrap();
    //println!("{:?}", mappings);

    let mut device_manager: DeviceManager = match DeviceManager::new(mappings, profiles) {
        Ok(value) => value,
        Err(err) => {
            println!("Failed to create manager: {}", err);
            return;
        }
    };

    loop {
        match device_manager.discover() {
            Ok(_) => {},
            Err(err) => {
                println!("Failed to discover devices: {}", err);
            }
        }
        thread::sleep(time::Duration::from_secs(4));
    }
}
