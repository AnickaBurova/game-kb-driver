use std::iter::Iterator;

use device_mapping::DeviceDigitalInput;
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
    pub fn generate_input(&mut self, keys: &[DeviceDigitalInput], buffer: &[u8]) -> Vec<Input> {
        let mut res = Vec::new();
        for (i, ref key) in keys.iter().enumerate() {
            let pressed_now = buffer[key.index as usize] & key.mask != 0;
            let pressed_already = self.pressed[i];
            if pressed_now && !pressed_already {
                res.push(Input::KeyDown(key.uid));
                self.pressed[i] = true;
            } else if !pressed_now && pressed_already {
                res.push(Input::KeyUp(key.uid));
                self.pressed[i] = false;
            }
        }
        res
    }
}
