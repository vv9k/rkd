use std::fs;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::{Path, PathBuf};
use std::thread;

const DEV_INP: &'static str = "/dev/input/";
pub const INPUT_DEVICE_LIST: &'static str = "/proc/bus/input/devices";
// When reading the INPUT_DEVICE_LIST file all keyboard devices have
// EV=120013
const KEYBOARD_INPUT_ID: &'static str = "120013";
const NAME_PREFIX: &'static str = "N: Name=\"";
const HANDLER_PREFIX: &'static str = "H: Handlers=";
// Each input event consist of exactly 24 bytes (see InputEvent struct)
const SIZE_OF_INPUT_EVENT: usize = mem::size_of::<InputEvent>();

const TOTAL_KEYS: u16 = 126;
const KEY_EV: u16 = 1;
const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
// Unknown key
const UK: &'static str = "<UK>";

#[rustfmt::skip]
const KEY_MAP: [&'static str; TOTAL_KEYS as usize] = [
    UK, "<ESC>", "1", "2", "3", "4", "5", "6", "7", "8",
    "9", "0", "-", "=", "<Backspace>", "<Tab>", "q", "w", "e", "r",
    "t", "y", "u", "i", "o", "p", "[", "]", "<Enter>", "<LCtrl>",
    "a", "s", "d", "f", "g", "h", "j", "k", "l", ";",
    "'", "`", "<LShift>", "\\", "z", "x", "c", "v", "b", "n",
    "m", ",", ".", "/", "<RShift>", "<KP*>", "<LAlt>", " ", "<CapsLock>", "<F1>",
    "<F2>", "<F3>", "<F4>", "<F5>", "<F6>", "<F7>", "<F8>", "<F9>", "<F10>",
    "<NumLock>", "<ScrollLock>", "<KP7>", "<KP8>", "<KP9>", "<KP->", "<KP4>", "<KP5>", "<KP6>", "<KP+>",
    "<KP1>", "<KP2>", "<KP3>", "<KP0>","<KP.>", UK, UK, UK, "<F11>", "<F12>",
    UK, UK, UK, UK, UK, UK, UK, "<KPEnter>", "<RCtrl>", "<KP/>",
    "<SysRq>", "<RAlt>", UK, "<Home>", "<Up>", "<PageUp>", "<Left>", "<Right>", "<End>", "<Down>",
    "<PageDown>", "<Insert>", "<Delete>", UK, UK, UK, UK, UK, UK, UK,
    "<Pause>", UK, UK, UK, UK, UK, "<Super>",
];
#[rustfmt::skip]
const SHIFT_KEY_MAP: [&'static str; TOTAL_KEYS as usize] = [
    UK, "<ESC>", "!", "@", "#", "$", "%", "^", "&", "*",
    "(", ")", "_", "+", "<Backspace>", "<Tab>", "Q", "W", "E", "R",
    "T", "Y", "U", "I", "O", "P", "{", "}", "<Enter>", "<LCtrl>",
    "A", "S", "D", "F", "G", "H", "J", "K", "L", ":",
    "\"", "~", "<LShift>", "|", "Z", "X", "C", "V", "B", "N",
    "M", "<", ">", "?", "<RShift>", "<KP*>", "<LAlt>", " ", "<CapsLock>", "<F1>",
    "<F2>", "<F3>", "<F4>", "<F5>", "<F6>", "<F7>", "<F8>", "<F9>", "<F10>",
    "<NumLock>", "<ScrollLock>", "<KP7>", "<KP8>", "<KP9>", "<KP->", "<KP4>", "<KP5>", "<KP6>", "<KP+>",
    "<KP1>", "<KP2>", "<KP3>", "<KP0>", "<KP.>", UK, UK, UK, "<F11>", "<F12>",
    UK, UK, UK, UK, UK, UK, UK, "<KPEnter>", "<RCtrl>", "<KP/>",
    "<SysRq>", "<RAlt>", UK, "<Home>", "<Up>", "<PageUp>", "<Left>", "<Right>", "<End>", "<Down>",
    "<PageDown>", "<Insert>", "<Delete>", UK, UK, UK, UK, UK, UK, UK,
    UK, UK, UK, UK, UK, UK, UK,
];

#[derive(Debug)]
struct Keyboard {
    handlers: Vec<String>,
    name: String,
}
impl Keyboard {
    // Parses a Keyboard object from a block read from INPUT_DEVICE_LIST file
    //
    // Example block:
    // I: Bus=0003 Vendor=046d Product=c33a Version=0111
    // N: Name="Logitech G413 Carbon Mechanical Gaming Keyboard"
    // P: Phys=usb-0000:0b:00.3-4/input0
    // S:
    // Sysfs=/devices/pci0000:00/0000:00:08.1/0000:0b:00.3/usb3/3-4/3-4:1.0/0003:046D:C33A.0001/input/input2
    // U: Uniq=188338553234
    // H: Handlers=sysrq kbd event2 leds
    // B: PROP=0
    // B: EV=120013
    // B: KEY=1000000000007 ff9f207ac14057ff febeffdfffefffff fffffffffffffffe
    // B: MSC=10
    // B: LED=7
    //
    // In this example the file with all interesting events will be located in:
    //   /dev/input/event2
    fn new(inp: &str) -> Keyboard {
        let lines = inp.split('\n');
        let mut handlers = Vec::new();
        let mut name = String::new();

        for line in lines {
            if line.starts_with(NAME_PREFIX) {
                name = line[NAME_PREFIX.len()..line.len() - 1].to_string();
            } else if line.starts_with(HANDLER_PREFIX) {
                handlers = line[HANDLER_PREFIX.len()..]
                    .split(' ')
                    // Only interested in event handlers
                    .filter(|h| h.starts_with("event"))
                    .map(|h| h.to_string())
                    .collect();
            }
        }

        Keyboard { handlers, name }
    }
    // Attempts to open all event handler files
    fn handlers(&self) -> Vec<Result<File, std::io::Error>> {
        self.handlers
            .iter()
            .map(|h| {
                let mut path = PathBuf::from(DEV_INP);
                path.push(h);
                File::open(path)
            })
            .collect()
    }
}

pub fn read_input_devices<P: AsRef<Path>>(f: P) {
    let device_list = fs::read_to_string(f.as_ref()).unwrap();
    let device_list = device_list.split("\n\n");

    // All devices with EV=120013
    let keyboards = device_list.filter(|dev| dev.contains(KEYBOARD_INPUT_ID));
    for k in keyboards {
        let x = Keyboard::new(&k);
        let handlers = x.handlers();
        for h in handlers {
            let handle = thread::spawn(|| {
                listen(h.expect("Error: failed to open input file"));
            });
            handle.join().expect("test");
        }
        println!("{:?}", x);
    }
}

pub fn listen(mut event_file: File) {
    let mut buf: [u8; SIZE_OF_INPUT_EVENT] = unsafe { mem::zeroed() };
    let mut shift_pressed = 0;
    loop {
        let num_of_bytes = event_file
            .read(&mut buf)
            .unwrap_or_else(|e| panic!("{}", e));

        if num_of_bytes != SIZE_OF_INPUT_EVENT {
            panic!("Error while reading from device file");
        }

        let event: InputEvent = unsafe { mem::transmute(buf) };
        if event.is_key_event() {
            if event.is_key_press() {
                if event.is_shift() {
                    shift_pressed += 1;
                }

                let text = get_key_text(event.code, shift_pressed);
            //if text == "<UK>" {
            //
            //println!(
            //"<UK>(code: {}, shift_pressed: {})",
            //event.code, shift_pressed
            //);
            //}
            //println!("{}", text);
            } else if event.is_key_release() {
                if event.is_shift() {
                    shift_pressed -= 1;
                }
            }
        }
    }
}

pub fn get_key_text(code: u16, shift_pressed: u8) -> &'static str {
    let arr = if shift_pressed != 0 {
        SHIFT_KEY_MAP
    } else {
        KEY_MAP
    };

    if code < TOTAL_KEYS {
        return arr[code as usize];
    } else {
        return UK;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InputEvent {
    _tv_sec: isize,  // from timeval struct
    _tv_usec: isize, // from timeval struct
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}
impl InputEvent {
    pub fn is_shift(&self) -> bool {
        self.code == KEY_LEFTSHIFT || self.code == KEY_RIGHTSHIFT
    }
    pub fn is_key_event(&self) -> bool {
        self.type_ == KEY_EV
    }
    pub fn is_key_press(&self) -> bool {
        self.value == KEY_PRESS
    }
    pub fn is_key_release(&self) -> bool {
        self.value == KEY_RELEASE
    }
}
