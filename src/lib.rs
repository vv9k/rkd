pub mod config;
pub mod input;
pub mod key;
use crate::config::*;
use crate::input::*;
use crate::key::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::clone::Clone;
use std::convert::TryInto;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
