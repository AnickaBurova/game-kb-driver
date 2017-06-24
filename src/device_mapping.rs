extern crate serde_yaml;
use std::io::Result;
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::convert::From;
use std::collections::HashMap;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceMap {
    pub name: String,
    pub key: Vec<Key>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub name: String,
    pub index: u8,
    pub mask: u8,
}

impl DeviceMap {
    pub fn read_file(file_path: &str) -> Result<HashMap<u32, DeviceMap>> {
        let mut file = File::open(file_path)?;
        match serde_yaml::from_reader(&mut file) {
            Ok(value) => Ok(value),
            Err(err) => Err(Error::new(ErrorKind::InvalidData, err)),
        }
    }
}

#[test]
fn test_serialise() {
    let devices = Device {
        name: "G13".to_owned(),
        vendor_id: 0x46d,
        product_id: 0xc21c,
        key: vec![
            Key {
                name: "G1".to_owned(),
                index: 3,
                mask: 0x01,
            },
        ]
    };
    println!("{:?}", devices);
    println!("{:?}",serde_yaml::to_string(&devices).unwrap());
}
