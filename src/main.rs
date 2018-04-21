#[macro_use]
extern crate clap;

extern crate log4rs;
#[macro_use]
extern crate log;

extern crate libusb;
extern crate libxdo;
extern crate yaml_rust;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

#[macro_use]
mod macros;
mod device_manager;
mod device_input;
mod device_mapping;
mod input;
mod map_input;
mod profile_definition;


use std::{thread, time};

use device_mapping::DeviceMaps;
use device_manager::DeviceManager;

use profile_definition::Profiles;


fn main() {
    use clap::{App, Arg};
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("log-config")
                 .long("log-config")
                 .help("Log configuration file")
                 .takes_value(true)
                 .default_value("log.yaml"))
        .arg(Arg::with_name("devices")
                 .long("devices")
                 .help("Devices definition file")
                 .takes_value(true)
                 .default_value("devices.yaml"))
        .arg(Arg::with_name("profiles")
                 .long("profiles")
                 .help("Profiles file")
                 .takes_value(true)
                 .default_value("profiles.yaml"))
        .get_matches();
    let _ = log4rs::init_file(&matches.value_of("log-config").unwrap(), Default::default())
        .unwrap();

    let mappings = DeviceMaps::new(matches.value_of("devices").unwrap().as_ref()).unwrap();
    let device_inputs = mappings.get_inputs();
    let profiles = Profiles::new(matches.value_of("profiles").unwrap().as_ref(), device_inputs).unwrap();
    info!("{:?}", profiles);
    //println!("{:?}", mappings);

    let mut device_manager: DeviceManager = match DeviceManager::new(mappings, profiles) {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to create manager: {}", err);
            return;
        }
    };

    loop {
        match device_manager.discover() {
            Ok(_) => {},
            Err(err) => {
                error!("Failed to discover devices: {}", err);
            }
        }
        thread::sleep(time::Duration::from_secs(4));
    }
}
