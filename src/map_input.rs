use std::iter::Iterator;

use device_mapping::{DeviceDigitalInput, DeviceAnalogInput};
use input::Input;

pub struct MapInput {
    pressed: Vec<bool>,
    analogs: Vec<u8>,
}


impl MapInput {
    pub fn new(num_digitals: usize, num_analogs: usize) -> MapInput {
        MapInput {
            pressed: vec![false; num_digitals],
            analogs: vec![0; num_analogs],
        }
    }
    pub fn generate_input(&mut self, digitals: &[DeviceDigitalInput], analogs: &[DeviceAnalogInput], buffer: &[u8]) -> Vec<Input> {
        let mut res = Vec::new();
        for (i, ref digital) in digitals.iter().enumerate() {
            let pressed_now = buffer[digital.index as usize] & digital.mask != 0;
            let pressed_already = self.pressed[i];
            if pressed_now && !pressed_already {
                res.push(Input::ButtonDown(digital.uid));
                self.pressed[i] = true;
            } else if !pressed_now && pressed_already {
                res.push(Input::ButtonUp(digital.uid));
                self.pressed[i] = false;
            }
        }
        for (i, ref analog) in analogs.iter().enumerate() {
            let current = buffer[analog.index as usize];
            if current != self.analogs[i] {
                let old = self.analogs[i];
                self.analogs[i] = current;
                res.push(Input::Axis(analog.uid, analog.convert(current), analog.convert(old)));
            }
        }

        res
    }
}
