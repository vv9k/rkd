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

const KEY_EV: u16 = 1;
const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;

#[derive(Debug)]
#[rustfmt::skip]
pub enum Key {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Dash,
    Tick,
    Eq_,
    Dot,
    Comma,
    Slash,
    SemiColon,
    Apostrophe,
    BackSlash,
    LSquareBracket, RSquareBracket,
    RAlt, LAlt,
    RCtrl, LCtrl,
    RShift, LShift,
    Super,
    Esc,
    Backspace,
    Return,
    Space,
    Tab,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Up, Down, Left, Right,
    UK
}
impl Key {
    pub fn from_code(code: u16) -> Self {
        use self::Key::*;
        match code {
            1 => Esc,
            2 => Num1,
            3 => Num2,
            4 => Num3,
            5 => Num4,
            6 => Num5,
            7 => Num6,
            8 => Num7,
            9 => Num8,
            10 => Num9,
            11 => Num0,
            12 => Dash,
            13 => Eq_,
            14 => Backspace,
            15 => Tab,
            16 => Q,
            17 => W,
            18 => E,
            19 => R,
            20 => T,
            21 => Y,
            22 => U,
            23 => I,
            24 => O,
            25 => P,
            26 => LSquareBracket,
            27 => RSquareBracket,
            28 => Return,
            29 => LCtrl,
            30 => A,
            31 => S,
            32 => D,
            33 => F,
            34 => G,
            35 => H,
            36 => J,
            37 => K,
            38 => L,
            39 => SemiColon,
            40 => Apostrophe,
            41 => Tick,
            42 => LShift,
            43 => BackSlash,
            44 => Z,
            45 => X,
            46 => C,
            47 => V,
            48 => B,
            49 => N,
            50 => M,
            51 => Comma,
            52 => Dot,
            53 => Slash,
            54 => RShift,
            56 => LAlt,
            57 => Space,
            59 => F1,
            60 => F2,
            61 => F3,
            62 => F4,
            63 => F5,
            64 => F6,
            65 => F7,
            66 => F8,
            67 => F9,
            68 => F10,
            69 => F11,
            70 => F12,
            97 => RCtrl,
            100 => RAlt,
            103 => Up,
            105 => Left,
            106 => Right,
            108 => Down,
            125 => Super,
            _ => UK,
        }
    }
}

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

                let k = event.as_enum();

                //if text == "<UK>" {
                //
                //println!(
                //"<UK>(code: {}, shift_pressed: {})",
                //event.code, shift_pressed
                //);
                //}
                println!("{:?} {}", k, event.code);
            } else if event.is_key_release() {
                let k = event.as_enum();
                println!("releasing {:?}", k);
                if event.is_shift() {
                    shift_pressed -= 1;
                }
            }
        }
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
        match self.as_enum() {
            Key::LShift | Key::RShift => true,
            _ => false,
        }
    }
    pub fn is_ctrl(&self) -> bool {
        match self.as_enum() {
            Key::LCtrl | Key::RCtrl => true,
            _ => false,
        }
    }
    pub fn is_alt(&self) -> bool {
        match self.as_enum() {
            Key::LAlt | Key::RAlt => true,
            _ => false,
        }
    }
    pub fn is_super(&self) -> bool {
        match self.as_enum() {
            Key::Super => true,
            _ => false,
        }
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
    pub fn as_enum(&self) -> Key {
        Key::from_code(self.code)
    }
}
