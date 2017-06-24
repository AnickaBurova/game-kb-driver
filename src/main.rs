extern crate libusb;
extern crate hex_utils;
extern crate libxdo;
extern crate yaml_rust;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

#[macro_use]
mod macros;
mod device_mapping;
mod device;


use std::error::{Error};
use hex_utils::{Format, xxd_str};
use std::time::Duration;
use libxdo::XDo;
use std::char;

use device_mapping::DeviceMap;


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
    let mut mappings = DeviceMap::read_file("devices.yaml").unwrap();
    println!("{:?}", mappings);

    let mut context = libusb::Context::new().unwrap();

    let mut handle = None;

    for mut device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();
        let key = ((device_desc.vendor_id() as u32) << 16) + (device_desc.product_id() as u32);

        match mappings.remove(&key) {
            Some(mapping) => {
            }
            None => (),
        };

        if device_desc.vendor_id() == 0x46d && device_desc.product_id() == 0xc21c {
            println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id());
            println!("Configurations: {}", device_desc.num_configurations());
            let cfg = device.active_config_descriptor().unwrap();
            println!("Config index: {}", cfg.number());
            for interface in cfg.interfaces() {
                println!("Interface index: {}", interface.number());
                for desc in interface.descriptors() {
                    println!("end points: {}", desc.num_endpoints());
                    for endpoint in desc.endpoint_descriptors() {
                        println!("{:?}", endpoint);
                        println!("address: {}", endpoint.address());
                        println!("direction: {:?}", endpoint.direction());
                        println!("transfer_type: {:?}", endpoint.transfer_type());
                        println!("sync_type: {:?}", endpoint.sync_type());
                        println!("usage_type: {:?}", endpoint.usage_type())
                    }
                }
            }
            match device.open() {
                Ok(h) => handle = Some(h),
                Err(err) => println!("Error: {}", err.description()),
            }
            break;
        }
    }
    match handle {
        Some(mut handle) => {
            let mut xdo = XDo::new(None).unwrap();
            println!("active config: {}", handle.active_configuration().unwrap());
            if handle.kernel_driver_active(0).unwrap() {
                let _ = handle.detach_kernel_driver(0).unwrap();
            }
            let _ = handle.claim_interface(0).unwrap();
            let mut buffer = [0u8;8];
            let mut pressed = [0u8;8];
            let mut depress = [0u8;8];
            let mut last = 0;
            let keys = ["alt+n","b","c","d","e","f","g","h","j","i"];
            loop {
                match handle.read_interrupt(129, &mut buffer, Duration::from_secs(60)) {
                    Ok(size) => {
                        //println!("received {} bytes: {:?}, {:08b} {:08b}",size, &buffer[0..size],buffer[3], buffer[4]); 
                        for b in &buffer {
                            print!("{:08b} ", b);
                        }
                        println!("");
                        //println!("{:08b} {:08b}",buffer[3], buffer[4]); 
                        let current = buffer[3];
                        let pressed_size = bits((current ^ last) & current, &mut pressed);
                        let depress_size = bits((current ^ last) & last, &mut depress);
                        last = current;
                        for i in 0..pressed_size {
                            let c = keys[pressed[i as usize] as usize];
                            let _ = xdo.send_keysequence_down(c,5);
                        }
                        for i in 0..depress_size {
                            let c = keys[depress[i as usize] as usize];
                            let _ = xdo.send_keysequence_up(c,5);
                        }

                        //let _ = xdo.enter_text("hello world", 5).unwrap();
                    }
                    Err(err) => {
                        println!("Error: {}", err.description());
                        break;
                    }
                }
            }
        }
        None => {
            println!("Cannot find G13");
        }
    }
}
