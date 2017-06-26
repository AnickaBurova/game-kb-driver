
use device_mapping::*;
use std::io::{Result, Error, ErrorKind};
use std::collections::HashMap;
use libusb::{self,Context, DeviceHandle};
use std::convert::From;
use std::sync::mpsc::{Sender};
use input::Input;
