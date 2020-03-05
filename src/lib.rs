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

pub fn listen(mut event_file: File, keybindings: Arc<Mutex<Keybindings>>) {
    let mut buf: [u8; SIZE_OF_INPUT_EVENT] = [0; SIZE_OF_INPUT_EVENT];
    let mut key_combination: Vec<Key> = Vec::new();
    loop {
        let num_of_bytes = event_file
            .read(&mut buf)
            .unwrap_or_else(|e| panic!("{}", e));

        if num_of_bytes != SIZE_OF_INPUT_EVENT {
            panic!("Error while reading from device file");
        }

        match InputEvent::new(&buf) {
            Ok(event) => {
                if event.is_key_event() {
                    if event.is_key_press() {
                        let k = event.as_enum();
                        trace!("Pressed {:?}", k);
                        key_combination.push(k);
                    } else if event.is_key_release() {
                        let k = event.as_enum();
                        trace!("Released {:?}", k);
                        let mut remove_idx = 0;
                        let mut remove = false;
                        for (i, key) in key_combination.iter().enumerate() {
                            if *key == k {
                                remove_idx = i;
                                remove = true;
                                break;
                            }
                        }
                        if remove {
                            key_combination.remove(remove_idx);
                        }
                    }
                    trace!("Current key combination: {:?}", key_combination);

                    match keybindings.lock() {
                        Ok(mut keybindings) => {
                            if let Some(exec) = keybindings.get_mut(&key_combination) {
                                info!("running cmd {:?}", exec);
                                exec.run().expect("failed to execute cmd");
                            }
                        }
                        Err(e) => error!("faild to aquire lock for keybindings - {}", e),
                    }
                }
            }
            Err(e) => error!("Error: failed parsing InputEvent - {}", e),
        }
    }
}
