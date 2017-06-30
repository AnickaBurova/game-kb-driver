extern crate serde_yaml;
use std::io::Result;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DeviceMaps {
    pub devices: HashMap<u32, DeviceMap>,
}
#[derive(Debug, Clone)]
pub struct DeviceMap {
    pub name: String,
    pub packet_size: u16,
    pub keys: Vec<DeviceButton>,
}

#[derive(Debug, Clone)]
pub struct DeviceButton {
    pub name: String,
    pub uid: u16,
    pub index: u8,
    pub mask: u8,
}

#[derive(Debug, Clone)]
pub struct DeviceAxis {
}

#[derive(Debug)]
pub enum DeviceInput {
    Key(String, String, u16),
    Axis(String, String, u16),
}


#[derive(Serialize, Deserialize)]
struct DeviceMapDefinition {
    pub name: String,
    pub packet_size: u16,
    pub keys: Option<Vec<DeviceKeyDefinition>>,
    pub bytes: Option<Vec<DeviceByteDefinition>>,
}

#[derive(Serialize, Deserialize)]
struct DeviceKeyDefinition {
    pub name: String,
    pub index: u8,
    pub mask: u8,
}

#[derive(Serialize, Deserialize)]
struct DeviceByteDefinition {
    pub index: u8,
    pub names: Vec<String>,
}

impl DeviceMapDefinition {
    fn read_file(file_path: &str) -> Result<HashMap<u32, DeviceMapDefinition>> {
        let mut file = File::open(file_path)?;
        match serde_yaml::from_reader(&mut file) {
            Ok(value) => Ok(value),
            Err(err) => Err(Error::new(ErrorKind::InvalidData, err)),
        }
    }
}

impl DeviceMaps {
    pub fn new(file_path: &str) -> Result<DeviceMaps> {
        let mut def = DeviceMapDefinition::read_file(&file_path)?;
        let mut uid = 0;
        let mut devices = HashMap::new();
        for (product_key, mapping) in def.drain() {
            let mut keys = Vec::new();
            let packet_size = mapping.packet_size;
            let name = mapping.name;
            match mapping.keys {
                Some(mut mkeys) => {
                    for key in mkeys.drain(..) {
                        keys.push(
                            DeviceButton {
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
                                keys.push(
                                    DeviceButton {
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
                    keys,
                });
        }
        Ok(DeviceMaps{
            devices
        })
    }

    pub fn get_inputs(&self) -> Vec<DeviceInput> {
        let mut res = Vec::new();
        for ref device in self.devices.values() {
            for key in &device.keys {
                res.push(DeviceInput::Key(device.name.to_owned(), key.name.to_owned(), key.uid));
            }
        }
        res
    }
}

#[test]
fn test_serialise() {
}
