extern crate serde_yaml;
use std::io::Result;
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::convert::From;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceMap {
    pub name: String,
    pub packet_size: u16,
    pub keys: Vec<DeviceKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceKey {
    pub name: String,
    pub uid: u16,
    pub index: u8,
    pub mask: u8,
}


#[derive( PartialEq, Serialize, Deserialize)]
struct DeviceMapDefinition {
    pub name: String,
    pub packet_size: u16,
    pub keys: Vec<DeviceKeyDefinition>,
}

#[derive( PartialEq, Serialize, Deserialize)]
struct DeviceKeyDefinition {
    pub name: String,
    pub index: u8,
    pub mask: u8,
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
            for key in mapping.keys.drain(..) {
                keys.push(
                    DeviceKey {
                        name: key.name,
                        uid,
                        index: key.index,
                        mask: key.mask,
                    });
                uid += 1;
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
