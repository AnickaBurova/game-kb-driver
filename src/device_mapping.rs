extern crate serde_yaml;
use std::io::Result;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

/// All mapped devices hashed by their vendor_id and product_id.
#[derive(Debug, Clone)]
pub struct DeviceMaps {
    /// All devices hashed by their (vendor_id << 8 + product_id)
    pub devices: HashMap<u32, DeviceMap>, 
}


/// Mapping of a device.
#[derive(Debug, Clone)]
pub struct DeviceMap {
    /// Name of the device.
    pub name: String,
    /// Size of the input stream from the device. This is used as a chceck if everything is ok.
    pub packet_size: u16,
    /// All the mapped digital inputs on the device.
    pub digitals: Vec<DeviceDigitalInput>,
}

/// Mapped digital input on a device. This have two states, pressed or not pressed.
#[derive(Debug, Clone)]
pub struct DeviceDigitalInput {
    /// Name of the digital input, this is used to map profile outputs to this digital.
    pub name: String,
    /// Unique id of the input, this is auto generated on fly and used as communication between threads.
    pub uid: u16,
    /// Index of the byte in the device input stream.
    pub index: u8,
    /// Bit mask representing this input's pressed state.
    pub mask: u8,
}

/// Mapped analog input on a device. This have an interval of current state.
#[derive(Debug, Clone)]
pub struct DeviceAnalogInput {
    /// Name of the input, this is used to map profile outputs to this input.
    pub name: String,
    /// Unique id of the input, this is auto generated on fly and used as communication between threads.
    pub uid: u16,
    /// Index of the byte in the device input stream.
    pub indices: u8,
    /// Interval of values to map the input byte from 0 to 255. (This will usually be -1 to +1)
    pub state: (f32, f32),
}

#[derive(Debug)]
pub enum DeviceInputUid {
    Digital(String, String, u16),
    Analog(String, String, u16),
}


// These structs are used to read define mapping in yaml files.

/// Device mapping read form yaml files.
#[derive(Serialize, Deserialize)]
struct DeviceMapDefinition {
    pub name: String, // Name of the device, the name is used to map profiles to this device.
    pub packet_size: u16, // Number of bytes in the input stream expected to read from the usb.
    pub digitals: Option<Vec<DeviceButtonDefinition>>, // Definition of individual digitals on the device mapped to individual bytes and mask.
    pub bytes: Option<Vec<DeviceByteDefinition>>, // Definition of individual bytes in the device input, mapped to digitals in bit order.
}

/// Individual button mapped to a byte in the input stream on a device.
#[derive(Serialize, Deserialize)]
struct DeviceButtonDefinition {
    pub name: String, // Name of the digital. This is used to map profile to this digital.
    pub index: u8, // Index of the byte in device input stream.
    pub mask: u8, // Bit mask which is set when this digital is pressed.
}

/// Individual byte in the device input stream mapped to max of 8 digitals in bit order.
#[derive(Serialize, Deserialize)]
struct DeviceByteDefinition {
    pub index: u8, // Index of the byte in the input stream.
    pub names: Vec<String>, // Names of the digitals in bit order.
}

impl DeviceMapDefinition {
    /// Creates the device mapping definition from yaml file.
    fn new(file_path: &str) -> Result<HashMap<u32, DeviceMapDefinition>> {
        let mut file = File::open(file_path)?;
        match serde_yaml::from_reader(&mut file) {
            Ok(value) => Ok(value),
            Err(err) => Err(Error::new(ErrorKind::InvalidData, err)),
        }
    }
}

impl DeviceMaps {
    /// Creates the device mapping from yaml file.
    pub fn new(file_path: &str) -> Result<DeviceMaps> {
        // read the file
        let mut def = DeviceMapDefinition::new(&file_path)?;
        // convert the definition in to list of digitals and axis with unique id mapped to device
        // name and digital name
        let mut uid = 0;
        let mut devices = HashMap::new();
        for (product_key, mapping) in def.drain() {
            let mut digitals = Vec::new();
            let packet_size = mapping.packet_size;
            let name = mapping.name;
            // convert individual digitals
            match mapping.digitals {
                Some(mut mdigitals) => {
                    for key in mdigitals.drain(..) {
                        digitals.push(
                            DeviceDigitalInput {
                                name: key.name,
                                uid,
                                index: key.index,
                                mask: key.mask,
                            });
                        uid += 1;
                    }
                }
                None => (),
            }
            // convert individual bytes
            match mapping.bytes {
                Some(mut mbytes) => {
                    for mut byte in mbytes.drain(..) {
                        let mut mask = 1u8;
                        let index = byte.index;
                        if byte.names.len() > 8 {
                            let msg = format!("Mapping for device: {} has invalid number of names in byte: {}", name, index);
                            return Err(Error::new(ErrorKind::InvalidData, msg));
                        }
                        for name in byte.names.drain(..) {
                            if &name != "-" {
                                digitals.push(
                                    DeviceDigitalInput {
                                        name,
                                        uid,
                                        index,
                                        mask,
                                    });
                                uid += 1;
                            }
                            mask <<= 1;
                        }
                    }
                }
                None => (),
            }
            devices.insert( product_key,
                DeviceMap {
                    name,
                    packet_size,
                    digitals,
                });
        }
        Ok(DeviceMaps{
            devices
        })
    }

    /// Returns list of all digital and analog inputs and their unique ids.
    pub fn get_inputs(&self) -> Vec<DeviceInputUid> {
        let mut res = Vec::new();
        for ref device in self.devices.values() {
            for digital in &device.digitals {
                res.push(DeviceInputUid::Digital(device.name.to_owned(), digital.name.to_owned(), digital.uid));
            }
        }
        res
    }
}
