use std::io::{self};
use std::fs::File;
use std::collections::HashMap;
use serde_yaml::{self};
use libxdo::XDo;

use device_mapping::DeviceInputUid;

#[derive(Serialize, Deserialize)]
struct ProfileDef {
    name: String,
    pattern: String,
    modes: Vec<ModeDef>,
}

#[derive(Serialize, Deserialize)]
struct ModeDef {
    name: String,
    devices: Vec<DeviceDef>,
}

#[derive(Serialize, Deserialize)]
struct DeviceDef {
    device: String,
    single: Vec<SingleDef>,
}

#[derive(Serialize, Deserialize)]
struct SingleDef {
    key: String,
    cmd: String,
}

pub struct Profiles {
    pub profiles: Vec<Profile>,
}

pub struct Profile {
    pub name: String,
    pub pattern: String,
    pub modes: Vec<Mode>,
}

pub struct Mode {
    pub name: String,
    // each input uid has two output actions, (down, up)
    pub output: Vec<(Action, Action)>,
}

#[derive(Debug, Clone)]
pub enum Action {
    // no operation on this action, this button is not mapped,
    // or for example multi macro has no up action,
    // because the whole macro will happen on down
    NoOp,
    KeyDown(String),
    KeyUp(String),
}

pub fn execute(xdo: &XDo, action: &Action) {
    match action {
        &Action::NoOp => (),
        &Action::KeyDown(ref s) => {
            xdo.send_keysequence_down(s,0).unwrap();
        }
        &Action::KeyUp(ref s) => {
            xdo.send_keysequence_up(s,0).unwrap();
        }
    }
}



impl Profiles {
    pub fn new(file_path: &str, device_inputs: Vec<DeviceInputUid>) -> io::Result<Profiles> {
        let mut file = File::open(file_path)?;
        let mut profiles_def: Vec<ProfileDef> = match serde_yaml::from_reader(&mut file) {
            Ok(value) => value,
            Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
        };

        let mut inputs_index = HashMap::new();
        for dev_inp in &device_inputs {
            match dev_inp {
                &DeviceInputUid::Digital(ref device, ref key, ref index) => {
                    inputs_index.insert((device,key), *index);
                }
                &DeviceInputUid::Analog(ref device, ref key, ref index) => {
                    inputs_index.insert((device,key), *index);
                }
            }
        }

        println!("dev_inputs: {:?}", device_inputs);
        let mut profiles = Vec::new();
        for mut profile_def in profiles_def.drain(..) {
            let name = profile_def.name;
            let pattern = profile_def.pattern;
            let mut modes = Vec::new();

            for mut mode_def in profile_def.modes.drain(..) {
                let name = mode_def.name;
                let mut output = vec![(Action::NoOp, Action::NoOp); device_inputs.len()];
                for mut device_def in mode_def.devices.drain(..) {
                    let device_name = device_def.device;
                    for single in device_def.single.drain(..) {
                        let hash = (&device_name, &single.key);
                        let index = *inputs_index.get(&hash).unwrap() as usize;
                        output[index] = (Action::KeyDown(single.cmd.to_owned()), Action::KeyUp(single.cmd));
                    }
                }
                modes.push(Mode {
                    name,
                    output,
                });
            }

            profiles.push( Profile {
                name,
                pattern,
                modes,
            });
        }
        Ok(Profiles{
            profiles,
        })
    }
}

