
use device_mapping::*;
use std::io::{Result, Error, ErrorKind};
use std::collections::HashMap;
use libusb::{self,Context, DeviceHandle};
use std::convert::From;
use std::sync::mpsc::{Sender};
use input::Input;

pub struct Device<'a,'b> {
    mapping: &'b DeviceMap,
    handle: DeviceHandle<'a>,
}

pub trait HasDevice {
    fn has_address(&self, u16) -> bool;
}

pub struct DeviceList {

}

impl DeviceList {
    pub fn add_device(&mut self, address: u16, ) {

    }
}

impl HasDevice for DeviceList {
    fn has_address(&self, address: u16) -> bool {
        false
    }
}

pub type DiscoveredDevice<'a,'b> = (DeviceHandle<'a>, &'b DeviceMap);
pub type DiscoveredDevices<'a,'b> = HashMap<u16, DiscoveredDevice<'a,'b>>;

pub fn discover_devices<'a, 'b>(usb_context: &'a mut Context, discovered: &HasDevice, mappings: &'b HashMap<u32, DeviceMap>) -> Result<Vec<(u16, Result<DiscoveredDevice<'a, 'b>>)>> {
    let mut result = Vec::new();
    Ok(result)
}

//impl<'a,'b> Device<'a,'b> {
    //fn discover(usb_context: &'a mut Context, mappings: &'b HashMap<u32, DeviceMap>, sender: &Sender<Input>) -> Result<Vec<((> {
        //Result<Vec<Result<Device<'a>>>> {
        //let mut devices = Vec::new();
        //for mut device in usb_context.devices().unwrap().iter() {
            //let device_desc = device.device_descriptor().unwrap();
            //let key = ((device_desc.vendor_id() as u32) << 16) + (device_desc.product_id() as u32);

            //match mappings.remove(&key) {
                //Some(mapping) => {
                    ////println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                        ////device.bus_number(),
                        ////device.address(),
                        ////device_desc.vendor_id(),
                        ////device_desc.product_id());
                    ////println!("Configurations: {}", device_desc.num_configurations());
                    //let cfg = device.active_config_descriptor().unwrap();
                    ////println!("Config index: {}", cfg.number());
                    ////let mut i_index = None;
                    //for interface in cfg.interfaces() {
                        //println!("Interface index: {}", interface.number());
                        //for desc in interface.descriptors() {
                            //println!("end points: {}", desc.num_endpoints());
                            //for endpoint in desc.endpoint_descriptors() {
                                //println!("{:?}", endpoint);
                                //println!("address: {}", endpoint.address());
                                //println!("direction: {:?}", endpoint.direction());
                                //println!("transfer_type: {:?}", endpoint.transfer_type());
                                //println!("sync_type: {:?}", endpoint.sync_type());
                                //println!("usage_type: {:?}", endpoint.usage_type())
                            //}
                        //}
                    //}
                    //let handle = iotry!(device.open());
                    ////if handle.kernel_driver_active(0).unwrap() {
                        ////let _ = handle.detach_kernel_driver(0).unwrap();
                    ////}
                    ////let _ = handle.claim_interface(0).unwrap();
                    //devices.push(
                        //Ok(Device {
                            //mapping,
                            //handle,
                        //}));
                //}
                //None => (),
            //};
        //}
        //Ok(devices)
    //}
//}
