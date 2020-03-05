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
    pub fn parse(&self) -> Result<Keybindings, Box<dyn std::error::Error>> {
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
        use self::Key::*;
        let keys: Vec<&str> = line.split('+').map(|k| k.trim()).collect();
        let mut parsed_keys = Vec::new();
        let mut was_shift = false;
        for (i, key) in keys.iter().enumerate() {
            match Key::from_str(*key, was_shift) {
                Ok(k) => {
                    // First token has to be a modifier key
                    if i == 0 {
                        match &k[..] {
                            k @ ([Shift, ..] | [Alt] | [Ctrl] | [Super]) => {
                                if k[0] == Shift && k.len() == 1 {
                                    was_shift = true;
                                }
                                k.iter().for_each(|key| parsed_keys.push(key.clone()))
                            }
                            _ => return None,
                        }
                    } else {
                        match &k[..] {
                            k @ [Shift, ..] if k.len() == 1 => {
                                if was_shift && k[0] == Shift {
                                    error!(
                                        "Failed parsing keybinding '{}' - double Shift modifier",
                                        line
                                    );
                                    return None;
                                } else {
                                    was_shift = false;
                                }
                                parsed_keys.push(k[0].clone());
                            }
                            keys => keys.iter().for_each(|key| parsed_keys.push(key.clone())),
                        }
                    }
                }
                Err(e) => {
                    error!("Failed paring keybinding '{}' - {}", line, e);
                    return None;
                }
            }
        }
        //ensure all elements are unique
        if Self::check_keys_unique(&parsed_keys) {
            Some(parsed_keys)
        } else {
            error!(
                "failed to parse keybinding '{:?}' - all keys have to be unique in a keybinding",
                &parsed_keys
            );
            None
        }
    }

    // Checks if all the keys are unique
    fn check_keys_unique(keys: &[Key]) -> bool {
        let mut unique = HashSet::new();
        for k in keys {
            if !unique.insert(k) {
                return false;
            }
        }
        true
    }
}
