#![feature(or_patterns)]
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

const DEV_INP_BY_ID: &'static str = "/dev/input/by-id";
const INPUT_DEVICE_LIST: &'static str = "/proc/bus/input/devices";
const KEYBOARD_INPUT_ID: &'static str = "120013";
const NAME_PREFIX: &'static str = "N: Name=\"";
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
                match k.handlers() {
                    Ok(handlers) => {
                        let mut thr_handles = Vec::new();
                        for h in handlers {
                            let _kb = kb.clone();
                            let handle = thread::spawn(|| {
                                listen(h.expect("failed to open input file"), _kb);
                            });
                            thr_handles.push((&k.name, handle));
                        }
                        for (kb, handle) in thr_handles {
                            handle.join().map_err(|_| {
                                error!("failed to join a thread handle for keyboard {}", kb)
                            });
                        }
                    }
                    Err(e) => {
                        error!("Failed to read handlers for keyboard {} - {}", k.name, e);
                        std::process::exit(1);
                    }
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
        match event_file.read(&mut buf) {
            Ok(num_of_bytes) => {
                if num_of_bytes != SIZE_OF_INPUT_EVENT {
                    error!("invalid input {:?}", &buf);
                    continue;
                }

                match InputEvent::new(&buf) {
                    Ok(event) => {
                        if event.is_key_event() {
                            if event.is_key_press() {
                                let k = event.as_enum();
                                trace!("Pressed {:?}, key_code: {}", k, event.code);
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
                                        exec.run().map_err(|e| {
                                            error!("failed to execute command - {}", e)
                                        });
                                    }
                                }
                                Err(e) => error!("faild to aquire lock for keybindings - {}", e),
                            }
                        }
                    }
                    Err(e) => error!("Error: failed parsing InputEvent - {}", e),
                }
            }
            Err(e) => {
                error!("invalid input event - {}", e);
            }
        }
    }
}
