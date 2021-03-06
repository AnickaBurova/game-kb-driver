use std::io::{Result, Error, ErrorKind};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::{thread};
use libxdo::XDo;

use libusb::{Context, Direction};

use device_mapping::{ DeviceMaps};
use input::Input;
use profile_definition::{Profiles};
use device_input::{self};


pub struct DeviceManager {
    context: Context,
    mapping: DeviceMaps,
    input_sender: Sender<Input>,
    finished_sender: Sender<u16>,
    finished_receiver: Receiver<u16>,
    mapped: Vec<u16>,
}

fn run_mappings(rcv: Receiver<Input>, profiles: Profiles, profile: Option<String>) {
    let xdo = XDo::new(None).unwrap();
    let (ref name, ref output) = match profile {
        None => {
            None
        }
        Some(name) => {
            let name = name.to_lowercase();
            let mut result = None;
            for ref profile in profiles.profiles.iter() {
                let pname = profile.name.to_lowercase();
                if pname.starts_with(&name) {
                    result = Some((&profile.name, &profile.modes[0].output));
                    break;
                }
            }
            result
        }
    }.unwrap_or((&profiles.profiles[0].name, &profiles.profiles[0].modes[0].output));
    info!("Profile selected: {}", name);
    for inp in rcv.iter() {
        // find name of the key
        match inp {
            Input::ButtonDown(uid) => {
                output[uid as usize].execute(&xdo, 1.0, 0.0);
            }
            Input::ButtonUp(uid) => {
                output[uid as usize].execute(&xdo, 0.0, 1.0);
            }
            Input::Axis(uid, value, old_value) => {
                output[uid as usize].execute(&xdo, value, old_value);
            }
        }
    }
}


impl DeviceManager {
    pub fn new(mapping: DeviceMaps, profiles: Profiles, profile: Option<&str>) -> Result<DeviceManager> {
        let context = iotry!(Context::new());
        let (input_sender, input_receiver) = mpsc::channel();
        let (finished_sender, finished_receiver) = mpsc::channel();
        let profile = profile.map(|s| s.to_owned());
        //let dev_maps = mapping.devices.values().map(|ref m| (*m).clone()).collect::<Vec<DeviceMap>>();
        thread::spawn(move || {
            run_mappings(input_receiver, profiles, profile);
        });
        Ok(DeviceManager {
            context, // context of usblib, which is used to find connected devices
            mapping, // definition of mapping raw data to keys and axes
            input_sender, // devices are sending input keys and axes using this channel
            finished_sender, // when a device is disconnected or some error, finishing thread will send its address through this
            finished_receiver, // when a device is disconnected or some error, finished threads addresses are received here
            mapped: Vec::new(), // addresses of already mapped devices
        })
    }

    pub fn discover(&mut self) -> Result<()> {
        trace!("Removing finished devices");
        // removing finished devices from mapped
        for address in self.finished_receiver.try_iter() {
            trace!("Device at {} has finished, removing it", address);
            self.mapped.retain(|&a| a != address);
        }
        trace!("Finding devices");
        // search for new devices, which are not yet mapped
        for device in iotry!(self.context.devices()).iter() {
            let address = ((device.bus_number() as u16) << 8) + (device.address() as u16);
            if self.mapped.contains(&address) {
                continue; // this address is already mapped
            }

            let device_desc = iotry!(device.device_descriptor());
            // create key of the device, mapping definition is hashed by the vendor and product id
            let key = ((device_desc.vendor_id() as u32) << 16) + (device_desc.product_id() as u32);

            let mapping = match self.mapping.devices.get(&key) {
                Some(ref mapping) => (*mapping).clone(),
                None => {
                    continue;
                }
            };
            // find input interface
            let cfg = iotry!(device.active_config_descriptor());
            let mut ok = false;
            for interface in cfg.interfaces() {
                for desc in interface.descriptors() {
                    for endpoint in desc.endpoint_descriptors() {
                        if endpoint.direction() == Direction::In && endpoint.max_packet_size() == mapping.packet_size {
                            ok = true;
                            break;
                        }
                    }
                }
            }

            if !ok {
                error!("Device {} has no compatible endpoint",mapping.name);
                continue;
            }

            self.mapped.push(address);

            let input_sender = self.input_sender.clone();
            let finished_sender = self.finished_sender.clone();
            let bus_number = device.bus_number();
            let dev_address = device.address();
            thread::spawn(move || {
                info!("Running devices at {}", address);
                match device_input::run(bus_number, dev_address, mapping, input_sender) {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Failed to run device input: {}", err);
                    }
                }
                finished_sender.send(address).unwrap();
            });
        }

        Ok(())
    }

}
