
use device_mapping::*;
use std::io::{Result, Error, ErrorKind};
use std::collections::HashMap;
use libusb::{self,Context, DeviceHandle};
use std::convert::From;

pub struct Device<'a> {
    mapping: DeviceMap,
    handle: DeviceHandle<'a>,
}



impl<'a> Device<'a> {
    fn discover(usb_context: &'a mut Context, mappings: &mut HashMap<u32, DeviceMap>) -> Result<Vec<Result<Device<'a>>>> {
        let mut devices = Vec::new();
        for mut device in usb_context.devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();
            let key = ((device_desc.vendor_id() as u32) << 16) + (device_desc.product_id() as u32);

            match mappings.remove(&key) {
                Some(mapping) => {
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
                    let handle = iotry!(device.open());
                    devices.push(
                        Ok(Device {
                            mapping,
                            handle,
                        }));
                }
                None => (),
            };
        }
        Ok(devices)
    }
}
