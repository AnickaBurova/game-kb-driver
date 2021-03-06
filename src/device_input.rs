
use std::io::{Result, Error, ErrorKind};
use std::sync::mpsc::{Sender};
use std::time::Duration;

use libusb::{self,Context, Direction, TransferType};

use device_mapping::DeviceMap;
use input::Input;
use map_input::MapInput;


pub fn run(bus_number: u8, address: u8, mapping: DeviceMap, input_sender: Sender<Input>) -> Result<()> {
    let context = iotry!(Context::new());
    let mut handle = None;
    let mut iet = None;
    for device in iotry!(context.devices()).iter() {
        if bus_number != device.bus_number() || address != device.address() {
            continue;
        }

        // find input interface
        let cfg = iotry!(device.active_config_descriptor());
        for interface in cfg.interfaces() {
            for desc in interface.descriptors() {
                for endpoint in desc.endpoint_descriptors() {
                    //if endpoint.direction() == Direction::In && endpoint.max_packet_size() == mapping.packet_size {
                    if endpoint.direction() == Direction::In {
                        iet = Some((interface.number(), endpoint.address(), endpoint.transfer_type()));
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
        (Some(mut handle), Some((i,e,t))) => {
            if iotry!(handle.kernel_driver_active(i)) {
                let _ = iotry!(handle.detach_kernel_driver(i));
            }
            let _ = iotry!(handle.claim_interface(i));
            let mut input_buffer = vec![0u8;mapping.packet_size as usize];
            let mut mapper = MapInput::new(mapping.digitals.len(), mapping.analogs.len());
            loop {
                match &t {
                    &TransferType::Interrupt => {
                        match handle.read_interrupt(e, &mut input_buffer, Duration::from_secs(4)) {
                            Ok(_) => {},
                            Err(libusb::Error::Timeout) => { continue; }
                            Err(err) => {
                                return Err(Error::new(ErrorKind::InvalidInput, err));
                            }
                        }
                    }
                    _ => {
                        let msg = format!("Incompatible transfer method: {:?}", t);
                        return Err(Error::new(ErrorKind::InvalidInput, msg));
                    }
                }
                let mut res = String::new();
                for b in &input_buffer {
                    res = format!("{}{:08b} ",res, b);
                }
                for inp in mapper.generate_input(&mapping.digitals, &mapping.analogs, &input_buffer) {
                    let _ = iotry!(input_sender.send(inp));
                }
                trace!("{}",res);
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
