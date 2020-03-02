pub mod config;
pub mod input;
pub mod key;
use crate::config::*;
use crate::input::*;
use crate::key::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::{error, info, trace};
use std::clone::Clone;
use std::convert::TryInto;
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::mem;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;

const DEV_INP: &'static str = "/dev/input/";
const INPUT_DEVICE_LIST: &'static str = "/proc/bus/input/devices";
// When reading the INPUT_DEVICE_LIST file all keyboard devices have
// EV=120013
const KEYBOARD_INPUT_ID: &'static str = "120013";
const NAME_PREFIX: &'static str = "N: Name=\"";
const HANDLER_PREFIX: &'static str = "H: Handlers=";
// Each input event consist of exactly 24 bytes (see InputEvent struct)
const SIZE_OF_INPUT_EVENT: usize = mem::size_of::<InputEvent>();
const SIZE_OF_ISIZE: usize = mem::size_of::<isize>();

const KEY_EV: u16 = 1;
const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;

pub fn run_rkd(keybindings: Keybindings) {
    info!("Starting rkd");
    trace!("{:?}", &keybindings);
    match read_input_devices() {
        Ok(keyboards) => {
            let kb = Arc::new(Mutex::new(keybindings));
            for k in keyboards {
                let handlers = k.handlers();
                let mut thr_handles = Vec::new();
                for h in handlers {
                    let _kb = kb.clone();
                    let handle = thread::spawn(|| {
                        listen(h.expect("Error: failed to open input file"), _kb);
                    });
                    thr_handles.push(handle);
                }
                for h in thr_handles {
                    h.join().expect("task failed successfully");
                }
            }
        }
        Err(e) => eprintln!(
            "Error: failed while reading '{}' file - {}",
            INPUT_DEVICE_LIST, e
        ),
    }
}
