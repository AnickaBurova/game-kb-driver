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
    singles: Vec<SinglesDef>,
    axes: Vec<AxisDef>,
}

#[derive(Serialize, Deserialize)]
struct SingleDef {
    button: String,
    cmd: String,
}

#[derive(Serialize, Deserialize)]
struct SinglesDef {
    button: String,
    index: u8,
    cmds: Vec<String>,
}



#[derive(Serialize, Deserialize)]
struct AxisDef {
    axis: String,
    simple: Vec<String>,
}


#[derive(Debug)]
pub struct Profiles {
    pub profiles: Vec<Profile>,
}

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub pattern: String,
    pub modes: Vec<Mode>,
}

#[derive(Debug)]
pub struct Mode {
    pub name: String,
    pub output: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum Action {
    NoOp,
    Key(String),
    Axis(String,String)
}



impl Action {
    pub fn execute(&self, xdo: &XDo, input: f32, old_input: f32) {
        match self {
            &Action::NoOp => (),
            &Action::Key(ref s) => {
                if input > 0.5 {
                    xdo.send_keysequence_down(s,0).unwrap();
                } else {
                    xdo.send_keysequence_up(s,0).unwrap();
                }
            }
            &Action::Axis(ref l, ref r) => {
                let lp = old_input < -0.5;
                let rp = old_input > 0.5;
                if lp && input >= -0.5 {
                    xdo.send_keysequence_up(l,0).unwrap();
                } else if !lp && input < -0.5 {
                    xdo.send_keysequence_down(l,0).unwrap();
                }
                if rp && input <= 0.5 {
                    xdo.send_keysequence_up(r,0).unwrap();
                } else if !rp && input > 0.5 {
                    xdo.send_keysequence_down(r,0).unwrap();
                }
            }
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
                &DeviceInputUid::Digital(ref device, ref button, ref index) => {
                    inputs_index.insert((device,button), *index);
                }
                &DeviceInputUid::Analog(ref device, ref button, ref index) => {
                    inputs_index.insert((device,button), *index);
                }
            }
        }

        trace!("dev_inputs: {:?}", device_inputs);
        let mut profiles = Vec::new();
        for mut profile_def in profiles_def.drain(..) {
            let name = profile_def.name;
            let pattern = profile_def.pattern;
            let mut modes = Vec::new();

            for mut mode_def in profile_def.modes.drain(..) {
                let name = mode_def.name;
                let mut output = vec![Action::NoOp; device_inputs.len()];
                for mut device_def in mode_def.devices.drain(..) {
                    let device_name = device_def.device;
                    for single in device_def.single.drain(..) {
                        let hash = (&device_name, &single.button);
                        let index = *inputs_index.get(&hash).unwrap() as usize;
                        output[index] = Action::Key(single.cmd.to_owned());
                    }
                    for mut singles in device_def.singles.drain(..) {
                        let prefix = singles.button;
                        let index = singles.index;
                        for (i,cmd) in singles.cmds.drain(..).enumerate() {
                            let name = format!("{}{}", prefix, index + (i as u8));
                            let hash = (&device_name, &name);
                            let index = *inputs_index.get(&hash).unwrap() as usize;
                            output[index] = Action::Key(cmd);
                        }
                    }
                    for mut axis in device_def.axes.drain(..) {
                        let hash = (&device_name, &axis.axis);
                        let index = *inputs_index.get(&hash).unwrap() as usize;
                        let right = axis.simple.pop().unwrap();
                        let left = axis.simple.pop().unwrap();
                        output[index] = Action::Axis(left, right);
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

