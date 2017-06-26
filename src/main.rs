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
mod device;
mod input;
mod map_input;


use std::error::{Error};
use std::{thread, time};
use hex_utils::{Format, xxd_str};
use std::time::Duration;
use libxdo::XDo;
use std::char;

use device_mapping::DeviceMap;
use device_manager::DeviceManager;


fn bits(mask: u8, indices: &mut [u8]) -> u8 {
    let mut val = mask;
    let mut size = 0;
    let mut current = 0;
    loop {
        if val == 0 {
            return size;
        }
        if val & 1 == 1 {
            indices[size as usize] = current;
            size += 1;
        }
        current += 1;
        val >>= 1;
    }
    return 0;
}

fn main() {
    let mappings = DeviceMap::read_file("devices.yaml").unwrap();
    println!("{:?}", mappings);

    let mut device_manager: DeviceManager = match DeviceManager::new(mappings) {
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
