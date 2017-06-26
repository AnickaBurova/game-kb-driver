

use device_mapping::DeviceKey;
use input::Input;

pub struct MapInput {
    pressed: Vec<bool>,
}


impl MapInput {
    pub fn new(num_keys: usize) -> MapInput {
        MapInput {
            pressed: vec![false; num_keys]
        }
    }
    pub fn generate_input(&mut self, keys: &Vec<DeviceKey>, buffer: &[u8]) -> Vec<Input> {
        let mut res = Vec::new();
        for ref key in keys {

        }
        res
    }
}
