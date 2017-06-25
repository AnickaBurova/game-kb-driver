use std::io::{Result, Error, ErrorKind};
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::{thread, time};
use rand::{self,Rng};
use rand::distributions::{IndependentSample, Range};

use libusb::{Context, Direction};

use device_mapping::DeviceMap;
use input::Input;
use device_input::DeviceInput;


pub struct DeviceManager {
    context: Context,
    mapping: HashMap<u32,DeviceMap>,
    input_sender: Sender<Input>,
    finished_sender: Sender<u16>,
    finished_receiver: Receiver<u16>,
    mapped: Vec<u16>,
}

fn run_mappings(rcv: Receiver<Input>) {
    for inp in rcv.iter() {
        println!("input: {:?}",inp);
    }
}


impl DeviceManager {
    pub fn new(mapping: HashMap<u32,DeviceMap>) -> Result<DeviceManager> {
        let context = iotry!(Context::new());
        let (input_sender, input_receiver) = mpsc::channel();
        let (finished_sender, finished_receiver) = mpsc::channel();
        thread::spawn(move || {
            run_mappings(input_receiver);
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
        println!("Removing finished devices");
        // removing finished devices from mapped
        for address in self.finished_receiver.try_iter() {
            println!("Device at {} has finished, removing it", address);
            self.mapped.retain(|&a| a != address);
        }
        println!("Finding devices");
        // search for new devices, which are not yet mapped
        for mut device in iotry!(self.context.devices()).iter() {
            let address = ((device.bus_number() as u16) << 8) + (device.address() as u16);
            if self.mapped.contains(&address) {
                continue; // this address is already mapped
            }

            let device_desc = iotry!(device.device_descriptor());
            // create key of the device, mapping definition is hashed by the vendor and product id
            let key = ((device_desc.vendor_id() as u32) << 16) + (device_desc.product_id() as u32);

            let mapping = match self.mapping.get(&key) {
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
                println!("Device {} has no compatible endpoint",mapping.name);
                continue;
            }

            //let handle = iotry!(device.open());
            self.mapped.push(address);

            let input_sender = self.input_sender.clone();
            let finished_sender = self.finished_sender.clone();
            let bus_number = device.bus_number();
            let dev_address = device.address();
            thread::spawn(move || {
                println!("Running devices at {}", address);
                match DeviceInput::run(bus_number, dev_address, mapping, input_sender) {
                    Ok(_) => {},
                    Err(err) => {
                        println!("Failed to run device input: {}", err);
                    }
                }
                //println!("active config: {}", handle.active_configuration().unwrap());
                //if handle.kernel_driver_active(0).unwrap() {
                    //let _ = handle.detach_kernel_driver(0).unwrap();
                //}
                //let _ = handle.claim_interface(0).unwrap();
                //let key = rand::random::<u16>();
                //println!("Key {} down", key);
                //match input_sender.send(Input::KeyDown(key)) {
                    //Ok(_) => {},
                    //Err(err) => {
                        //println!("Failed to send: {}", err);
                    //}
                //}
                //let mut rng = rand::thread_rng();
                //let sleep_interval = Range::new(4,10);
                //let sleep = sleep_interval.ind_sample(&mut rng);
                //thread::sleep(time::Duration::from_secs(sleep));
                //println!("Key {} up", key);
                //match input_sender.send(Input::KeyUp(key)) {
                    //Ok(_) => {},
                    //Err(err) => {
                        //println!("Failed to send: {}", err);
                    //}
                //}
                finished_sender.send(address).unwrap();
            });
        }

        Ok(())
    }

}
