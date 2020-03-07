use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Exec {
    cmd: Command,
    args: Vec<String>,
    _first_launch: bool,
}
impl Exec {
    pub fn new<S: AsRef<str>>(cmd: S) -> Option<Exec> {
        trace!("Creating exec instance from '{}'", cmd.as_ref());
        let mut _cmd = cmd.as_ref().split(' ');
        if let Some(cmd) = _cmd.next() {
            return Some(Exec {
                cmd: Command::new(cmd),
                args: _cmd.map(|arg| arg.to_string()).collect(),
                _first_launch: true,
            });
        }
        None
    }
    pub fn run(&mut self) -> io::Result<Child> {
        if self._first_launch {
            self._first_launch = false;
            self.cmd.args(&self.args).spawn()
        } else {
            self.cmd.spawn()
        }
    }
}

pub type Keybindings = HashMap<Vec<Key>, Exec>;

#[derive(Clone)]
pub struct Cfg<P: AsRef<Path>> {
    cfg_file: P,
}
impl<P: AsRef<Path>> Cfg<P> {
    pub fn new(cfg_file: P) -> Self {
        Self { cfg_file }
    }
    pub fn parse(&self) -> io::Result<Keybindings> {
        let file_content = fs::read_to_string(self.cfg_file.as_ref())?;
        let lines = file_content.split('\n');
        let mut keybindings = HashMap::new();
        let mut current_kb = Vec::new();
        let mut current_cmd = String::new();
        let mut was_keybinding = false;
        let mut was_cmd = false;
        lines.for_each(|line| {
            if Self::is_keybinding(&line) && !was_keybinding {
                if let Some(kb) = Self::parse_keybinding(&line) {
                    current_kb = kb;
                    was_keybinding = true;
                    if was_cmd {
                        was_cmd = false;
                    }
                }
            } else if Self::is_cmd(&line) && !was_cmd && was_keybinding {
                current_cmd = line.trim().to_string();
                if let Some(exec) = Exec::new(&current_cmd) {
                    keybindings.insert(current_kb.clone(), exec);
                }
                current_kb.clear();
                current_cmd.clear();
                was_cmd = true;
                was_keybinding = false;
            }
        });
        info!("{:?}", keybindings);
        Ok(keybindings)
    }

    pub fn is_keybinding(line: &str) -> bool {
        !(line.starts_with(' ') || line.starts_with('\t')) && line != ""
    }
    pub fn is_cmd(line: &str) -> bool {
        line.starts_with(' ') || line.starts_with('\t')
    }

    pub fn parse_keybinding(line: &str) -> Option<Vec<Key>> {
        trace!("parsing keybinding from {}", line);
        use self::Key::*;
        let keys: Vec<&str> = line.split('+').map(|k| k.trim()).collect();
        let mut parsed_keys = HashSet::new();

        for (i, key) in keys.iter().enumerate() {
            let k = Key::from_str(*key);
            // First token has to be a modifier key
            if i == 0 {
                match &k[..] {
                    k @ ([Shift, ..] | [Alt] | [Ctrl] | [Super]) => {
                        for key in k {
                            parsed_keys.insert(key.clone());
                        }
                    }
                    k
                    @
                    ([XF86AudioMute]
                    | [XF86AudioNext]
                    | [XF86AudioPlay]
                    | [XF86AudioPrev]
                    | [XF86AudioStop]
                    | [XF86AudioLowerVolume]
                    | [XF86AudioRaiseVolume]) => {
                        parsed_keys.insert(k[0].clone());
                    }
                    _ => {
                        error!("failed to parse keybinding '{}' - first key has to be a modifier (Alt | Shift | Ctrl | Super)", line);
                        return None;
                    }
                }
            } else {
                for key in k {
                    if !parsed_keys.insert(key.clone()) {
                        error!(
                            "failed to parse keybinding '{}' - all keys have to be unique in a keybinding",
                            line
                        );
                        return None;
                    }
                }
            }
        }
        Some(parsed_keys.iter().map(|k| *k).collect())
    }

    fn is_valid_keybinding(keys: &[Key]) -> bool {
        // This function should check if each keybinding consists
        // of at least one modifier key and one other key

        unimplemented!();
    }

    fn has_modifiers_after_key(keys: &[Key]) -> bool {
        // This function should check if there are modifier keys
        // after action key.
        //
        // For example:
        // [Shift, k, Ctrl] should return false

        unimplemented!();
    }
}
