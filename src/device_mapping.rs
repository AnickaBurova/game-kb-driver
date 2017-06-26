extern crate serde_yaml;
use std::io::Result;
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::convert::From;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DeviceMap {
    pub name: String,
    pub packet_size: u16,
    pub keys: Vec<DeviceKey>,
}

#[derive(Debug, Clone)]
pub struct DeviceKey {
    pub name: String,
    pub uid: u16,
    pub index: u8,
    pub mask: u8,
}

#[derive(Debug, Clone)]
pub struct DeviceAxis {
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

impl DeviceMap {
    pub fn read_file(file_path: &str) -> Result<HashMap<u32, DeviceMap>> {
        let mut def = DeviceMapDefinition::read_file(&file_path)?;
        let mut uid = 0;
        let mut res = HashMap::new();
        for (product_key, mut mapping) in def.drain() {
            let mut keys = Vec::new();
            let packet_size = mapping.packet_size;
            let name = mapping.name;
            match mapping.keys {
                Some(mut mkeys) => {
                    for key in mkeys.drain(..) {
                        keys.push(
                            DeviceKey {
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
                                    DeviceKey {
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
            res.insert( product_key,
                DeviceMap {
                    name,
                    packet_size,
                    keys,
                });
        }
        Ok(res)
    }
}

#[test]
fn test_serialise() {
}
