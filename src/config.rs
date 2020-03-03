use super::*;

#[derive(Debug)]
pub struct Exec {
    cmd: Command,
    args: Vec<String>,
    _first_launch: bool,
}
impl Exec {
    pub fn new<S: AsRef<str>>(cmd: S) -> Option<Exec> {
        let mut _cmd = cmd.as_ref().split(' ');
        if let Some(cmd) = _cmd.next() {
            trace!("{:?}", _cmd);
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

#[derive(Debug)]
pub struct Keybinding {
    pub key_combination: Vec<Key>,
    pub exec: Exec,
}
impl Keybinding {
    fn new(key_combination: &[Key], exec: Exec) -> Self {
        Self {
            key_combination: key_combination.to_vec(),
            exec,
        }
    }
}
pub type Keybindings = Vec<Keybinding>;

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
        let mut keybindings = Vec::new();
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
                    keybindings.push(Keybinding::new(&current_kb, exec));
                }
                current_kb.clear();
                current_cmd.clear();
                was_cmd = true;
                was_keybinding = false;
            }
        });
        println!("{:?}", keybindings);
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
        for (i, key) in keys.iter().enumerate() {
            let k = Key::from(*key);
            let parsed_key = if i == 0 {
                match k {
                    Alt | Shift | Ctrl | Super => k,
                    key => {
                        error!(
                            "Unsupported key '{:?}' as modifier. Ignoring keybinding {}",
                            key, line
                        );
                        return None;
                    }
                }
            } else {
                k
            };

            parsed_keys.push(parsed_key);
        }
        Some(parsed_keys)
    }
}
