
use std::io::{Result, Error, ErrorKind};
use std::sync::mpsc::{Sender};

use libusb::{Context, Direction, TransferType};

use device_mapping::DeviceMap;
use input::Input;


pub struct DeviceInput {
}

impl DeviceInput {
    pub fn run(bus_number: u8, address: u8, mapping: DeviceMap, input_sender: Sender<Input>) -> Result<()> {
        let context = iotry!(Context::new());
        let mut handle = None;
        let mut iet = None;
        for mut device in iotry!(context.devices()).iter() {
            if bus_number != device.bus_number() || address != device.address() {
                continue;
            }

            let device_desc = iotry!(device.device_descriptor());
            // find input interface
            let cfg = iotry!(device.active_config_descriptor());
            for interface in cfg.interfaces() {
                for desc in interface.descriptors() {
                    for endpoint in desc.endpoint_descriptors() {
                        if endpoint.direction() == Direction::In && endpoint.max_packet_size() == mapping.packet_size {
                            iet = Some((interface.number(), endpoint.number(), endpoint.transfer_type()));
                            break;
                        }
                    }
                }
            }

            if iet.is_none() {
                let msg = format!("Device {} has no compatible endpoint",mapping.name);
                return Err(Error::new(ErrorKind::InvalidInput, msg));
            }
            handle = match device.open() {
                Ok(handle) => Some(handle),
                Err(err) => return Err(Error::new(ErrorKind::InvalidInput, err)),
            };
            break;
        }
        match (handle, iet) {
            (Some(handle), Some((i,e,t))) => {
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
