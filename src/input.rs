use super::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::clone::Clone;
use std::convert::TryInto;
use std::io::Cursor;
use std::sync::Arc;

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

#[derive(Debug, PartialEq)]
pub struct Keyboard {
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

pub fn run_rdk(keybindings: Keybindings) {
    let keyboards = read_input_devices(INPUT_DEVICE_LIST);
    let kb = Arc::new(keybindings);
    for k in keyboards {
        let handlers = k.handlers();
        for h in handlers {
            let _kb = kb.clone();
            let handle = thread::spawn(|| {
                listen(h.expect("Error: failed to open input file"), _kb);
            });
            handle.join().expect("test");
        }
        println!("{:?}", k);
    }
}

pub fn read_input_devices<P: AsRef<Path>>(f: P) -> Vec<Keyboard> {
    let device_list = fs::read_to_string(f.as_ref()).unwrap();
    let device_list = device_list.split("\n\n");

    // All devices with EV=120013
    device_list
        .filter(|dev| dev.contains(KEYBOARD_INPUT_ID))
        .map(|k| Keyboard::new(&k))
        .collect()
}

pub fn listen(mut event_file: File, keybindings: Arc<Keybindings>) {
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
                        key_combination.push(k);
                    } else if event.is_key_release() {
                        let mut remove_idx = 0;
                        let mut remove = false;
                        for (i, key) in key_combination.iter().enumerate() {
                            if *key == event.as_enum() {
                                remove_idx = i;
                                remove = true;
                                break;
                            }
                        }
                        if remove {
                            key_combination.remove(remove_idx);
                        }
                    }
                    for kb in keybindings.iter() {
                        if kb.key_combination == key_combination {
                            println!("{}", kb.cmd);
                        }
                    }
                    println!("current combination: {:?}", &key_combination);
                }
            }
            Err(e) => eprintln!("Error - {}", e),
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
    pub fn new(buf: &[u8]) -> Result<InputEvent, std::io::Error> {
        let mut rdr = Cursor::new(&buf);
        Ok(InputEvent {
            _tv_sec: rdr.read_int::<LittleEndian>(8)?.try_into().unwrap(),
            _tv_usec: rdr.read_int::<LittleEndian>(8)?.try_into().unwrap(),
            type_: rdr.read_u16::<LittleEndian>()?,
            code: rdr.read_u16::<LittleEndian>()?,
            value: rdr.read_i32::<LittleEndian>()?,
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_keyboard() {
        let kb_txt = "I: Bus=0003 Vendor=046d Product=c33a Version=0111
N: Name=\"Logitech G413 Carbon Mechanical Gaming Keyboard\"
P: Phys=usb-0000:0b:00.3-4/input0
S:
Sysfs=/devices/pci0000:00/0000:00:08.1/0000:0b:00.3/usb3/3-4/3-4:1.0/0003:046D:C33A.0001/input/input2
U: Uniq=188338553234
H: Handlers=sysrq kbd event2 leds
B: PROP=0
B: EV=120013
B: KEY=1000000000007 ff9f207ac14057ff febeffdfffefffff fffffffffffffffe
B: MSC=10
B: LED=7";
        let kb = Keyboard {
            handlers: vec!["event2".to_string()],
            name: "Logitech G413 Carbon Mechanical Gaming Keyboard".to_string(),
        };
        let parsed_kb = Keyboard::new(&kb_txt);
        assert_eq!(kb, parsed_kb);
    }
}
