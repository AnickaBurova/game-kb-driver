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
    /// All the mapped analog inputs on the device.
    pub analogs: Vec<DeviceAnalogInput>,
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
    pub index: u8,
    /// Interval of values to map the input byte from 0 to 255. (This will usually be -1 to +1)
    pub output: (f32, f32),
}

impl DeviceAnalogInput {
    /// Convert input value to output interval.
    pub fn convert(&self, value: u8) -> f32 {
        let value = value as f32;
        value * (self.output.1 - self.output.0) / 256.0 + self.output.0
    }
}

#[test]
fn test_analog_convert() {
    let input = DeviceAnalogInput { name: "Test".to_owned(), uid: 0, index: 0, output: (-1.0, 1.0) };

    for i in 0..256 {
        println!("{} = {}", i, input.convert(i as u8));
    }
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
    /// Name of the device, the name is used to map profiles to this device.
    pub name: String,
    /// Number of bytes in the input stream expected to read from the usb.
    pub packet_size: u16,
    /// Definition of individual digitals on the device mapped to individual bytes and mask
    pub digitals: Option<Vec<DeviceButtonDefinition>>,
    /// Definition of individual bytes in the device input, mapped to digitals in bit order.
    pub bytes: Option<Vec<DeviceByteDefinition>>,
    /// Definition of individual analog inputs on the divece mapped to individual bytes.
    pub analogs: Option<Vec<DeviceAnalogDefinition>>,
}

/// Individual button mapped to a byte in the input stream on a device.
#[derive(Serialize, Deserialize)]
struct DeviceButtonDefinition {
    /// Name of the digital.
    pub name: String,
    /// Index of the byte in device input stream.
    pub index: u8,
    /// Bit mask which is set when this digital is pressed.
    pub mask: u8,
}

/// Individual byte in the device input stream mapped to max of 8 digitals in bit order.
#[derive(Serialize, Deserialize)]
struct DeviceByteDefinition {
    /// Index of the byte in the input stream.
    pub index: u8,
    /// Names of the digitals in bit order.
    pub names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct DeviceAnalogDefinition {
    /// Name of the analog input.
    pub name: String,
    /// Index of the byte in device input stream.
    pub index: u8,
    /// Interval of values to map the input byte from 0 to 255. (This will usually be -1 to +1)
    pub output: (f32, f32),
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
                    for digital in mdigitals.drain(..) {
                        digitals.push(
                            DeviceDigitalInput {
                                name: digital.name,
                                uid,
                                index: digital.index,
                                mask: digital.mask,
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
            let mut analogs = Vec::new();
            match mapping.analogs {
                Some(mut manalogs) => {
                    for analog in manalogs.drain(..) {
                        analogs.push( DeviceAnalogInput {
                            name: analog.name,
                            uid,
                            index: analog.index,
                            output: analog.output,
                        });
                        uid += 1;
                    }
                }
                None => (),
            }
            devices.insert( product_key,
                DeviceMap {
                    name,
                    packet_size,
                    digitals,
                    analogs,
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
            for analog in &device.analogs {
                res.push(DeviceInputUid::Analog(device.name.to_owned(), analog.name.to_owned(), analog.uid));
            }
        }
        res
    }
}
