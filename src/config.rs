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
        let keys: Vec<&str> = line.split('+').map(|k| k.trim()).collect();
        let mut parsed_keys = HashSet::new();

        for key in keys {
            let k = Key::from_str(key);
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
        if Self::is_valid_keybinding(&parsed_keys.iter().map(|k| *k).collect::<Vec<Key>>()) {
            Some(parsed_keys.iter().map(|k| *k).collect())
        } else {
            error!("invalid keybinding {:?}", &parsed_keys);
            None
        }
    }

    fn is_valid_keybinding(keys: &[Key]) -> bool {
        let mut is_valid = true;
        let mut keys_iter = keys.iter();

        match keys_iter.next() {
            Some(first) => {
                if first.is_modifier() {
                    if keys.len() >= 2 {
                        for (i, key) in keys[1..].iter().enumerate() {
                            if i + 1 == keys.len() - 1 {
                                // if the last key is not an action key the keybinding is invalid
                                is_valid = key.is_action();
                            } else {
                                if !key.is_modifier() {
                                    return false;
                                } else {
                                    continue;
                                }
                            }
                        }
                    } else {
                        return false;
                    }
                } else {
                    // If its not a mod key then its only valid
                    // if its a media control key like XF86AudioRaiseVolume
                    return first.is_media_control() && keys.len() == 1;
                }
            }
            None => return false,
        }

        is_valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_keybindings() {
        use self::config::Cfg;
        use self::Key::*;
        let keybindings = vec![
            (vec![Shift, K], true),
            (vec![Shift], false),
            (vec![Ctrl, Shift, Num2], true),
            (vec![Super, Return], true),
            (vec![Super, A, B], false),
            (vec![Super, A, Ctrl], false),
            (vec![Alt, F1], true),
            (vec![A], false),
            (vec![XF86AudioRaiseVolume, XF86AudioStop], false),
            (vec![A, XF86AudioStop], false),
            (vec![A, Shift], false),
            (vec![XF86AudioPlay], true),
        ];

        keybindings.iter().for_each(|kb| {
            println!("validating {:?}", kb);
            assert_eq!(Cfg::<&Path>::is_valid_keybinding(&kb.0), kb.1)
        });
    }
}
