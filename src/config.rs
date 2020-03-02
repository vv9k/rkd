use super::*;

#[derive(Debug)]
pub struct Keybinding {
    pub key_combination: Vec<Key>,
    pub cmd: String,
}
impl Keybinding {
    fn new<S: AsRef<str>>(key_combination: &[Key], cmd: S) -> Self {
        Self {
            key_combination: key_combination.to_vec(),
            cmd: cmd.as_ref().to_string(),
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
                current_kb = Self::parse_keybinding(&line);
                was_keybinding = true;
                if was_cmd {
                    was_cmd = false;
                }
            } else if Self::is_cmd(&line) && !was_cmd && was_keybinding {
                current_cmd = line.trim().to_string();
                keybindings.push(Keybinding::new(&current_kb, &current_cmd));
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

    pub fn parse_keybinding(line: &str) -> Vec<Key> {
        let keys: Vec<&str> = line.split('+').map(|k| k.trim()).collect();
        keys.iter().map(|k| Key::from(*k)).collect()
    }
    pub fn parse_cmd(line: &str) {
        println!("cmd: {}", &line);
    }
}
