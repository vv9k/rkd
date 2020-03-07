use super::*;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
#[rustfmt::skip]
pub enum Key {
    // Shift mapped keys
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
    Equal,
    Dot,
    Comma,
    Slash,
    SemiColon,
    Apostrophe,
    BackSlash,
    LSquareBracket,
    RSquareBracket,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Unique keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Up, Down, Left, Right,
    RAlt, LAlt, Alt,
    RCtrl, LCtrl, Ctrl,
    RShift, LShift, Shift,
    Super,
    Esc,
    Backspace,
    Return,
    Space,
    Tab,
    UK,

    XF86AudioRaiseVolume, XF86AudioLowerVolume,
    XF86AudioMute,
    XF86AudioPrev, XF86AudioNext,
    XF86AudioPlay, XF86AudioStop,

    // Find out the exact key codes of these
    XF86MonBrightnessUp,
    XF86MonBrightnessDown,
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
            13 => Equal,
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
            29 => Ctrl, // LCtrl
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
            42 => Shift, // LShift
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
            54 => Shift, // RShift
            56 => Alt,   // LAlt
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
            87 => F11,
            88 => F12,
            97 => Ctrl, // RCtrl
            100 => Alt, // RAlt
            103 => Up,
            105 => Left,
            106 => Right,
            108 => Down,
            113 => XF86AudioMute,
            114 => XF86AudioLowerVolume,
            115 => XF86AudioRaiseVolume,
            163 => XF86AudioNext,
            164 => XF86AudioPlay,
            165 => XF86AudioPrev,
            166 => XF86AudioStop,
            125 => Super,
            _ => UK,
        }
    }
    pub fn from_str(token: &str) -> Vec<Self> {
        trace!("parsing token {}", token);
        use self::Key::*;
        let mut parsed_keys = Vec::new();
        let lowercased_token = token.to_lowercase();
        // all keys that dont have their coresponding shift version
        let is_unique = match lowercased_token.as_ref() {
            "alt"
            | "ctrl"
            | "super"
            | "shift"
            | "esc"
            | "backspace"
            | "return"
            | "space"
            | "tab"
            | "f1"
            | "f2"
            | "f3"
            | "f4"
            | "f5"
            | "f6"
            | "f7"
            | "f8"
            | "f9"
            | "f10"
            | "f11"
            | "f12"
            | "up"
            | "down"
            | "left"
            | "right"
            | "xf86audiomute"
            | "xf86audio
lowervolume"
            | "xf86audioraisevolume"
            | "xf86audionext"
            | "xf86audioplay"
            | "xf86audioprev"
            | "xf86audiostop" => true,
            _ => false,
        };
        if is_unique {
            match lowercased_token.as_ref() {
                "alt" => parsed_keys.push(Alt),
                "ctrl" => parsed_keys.push(Ctrl),
                "shift" => parsed_keys.push(Shift),
                "super" => parsed_keys.push(Super),
                "esc" => parsed_keys.push(Esc),
                "backspace" => parsed_keys.push(Backspace),
                "return" => parsed_keys.push(Return),
                "space" => parsed_keys.push(Space),
                "tab" => parsed_keys.push(Tab),
                "f1" => parsed_keys.push(F1),
                "f2" => parsed_keys.push(F2),
                "f3" => parsed_keys.push(F3),
                "f4" => parsed_keys.push(F4),
                "f5" => parsed_keys.push(F5),
                "f6" => parsed_keys.push(F6),
                "f7" => parsed_keys.push(F7),
                "f8" => parsed_keys.push(F8),
                "f9" => parsed_keys.push(F9),
                "f10" => parsed_keys.push(F10),
                "f11" => parsed_keys.push(F11),
                "f12" => parsed_keys.push(F12),
                "up" => parsed_keys.push(Up),
                "down" => parsed_keys.push(Down),
                "left" => parsed_keys.push(Left),
                "right" => parsed_keys.push(Right),
                "xf86audiomute" => parsed_keys.push(XF86AudioMute),
                "xf86audiolowervolume" => parsed_keys.push(XF86AudioLowerVolume),
                "xf86audioraisevolume" => parsed_keys.push(XF86AudioRaiseVolume),
                "xf86audionext" => parsed_keys.push(XF86AudioNext),
                "xf86audioplay" => parsed_keys.push(XF86AudioPlay),
                "xf86audioprev" => parsed_keys.push(XF86AudioPrev),
                "xf86audiostop" => parsed_keys.push(XF86AudioStop),
                // We exhaust all the options so this branch is unreachable
                _ => unreachable!(),
            }
        } else {
            // All shift modified tokens are single characters
            if let Some(ch) = token.chars().next() {
                let is_shift_modified = match ch {
                    '!'
                    | '@'
                    | '#'
                    | '$'
                    | '%'
                    | '^'
                    | '&'
                    | '*'
                    | '('
                    | ')'
                    | '_'
                    | '+'
                    | '<'
                    | '>'
                    | '?'
                    | ':'
                    | '\"'
                    | '|'
                    | '{'
                    | '}'
                    | '~'
                    | 'A'..='Z' => true,
                    _ => false,
                };
                if is_shift_modified {
                    parsed_keys.push(Shift);
                }
                match ch {
                    '`' | '~' => parsed_keys.push(Tick),
                    '0' | ')' => parsed_keys.push(Num0),
                    '1' | '!' => parsed_keys.push(Num1),
                    '2' | '@' => parsed_keys.push(Num2),
                    '3' | '#' => parsed_keys.push(Num3),
                    '4' | '$' => parsed_keys.push(Num4),
                    '5' | '%' => parsed_keys.push(Num5),
                    '6' | '^' => parsed_keys.push(Num6),
                    '7' | '&' => parsed_keys.push(Num7),
                    '8' | '*' => parsed_keys.push(Num8),
                    '9' | '(' => parsed_keys.push(Num9),
                    '-' | '_' => parsed_keys.push(Dash),
                    '=' | '+' => parsed_keys.push(Equal),
                    '.' | '>' => parsed_keys.push(Dot),
                    ',' | '<' => parsed_keys.push(Comma),
                    '/' | '?' => parsed_keys.push(Slash),
                    ';' | ':' => parsed_keys.push(SemiColon),
                    '\'' | '"' => parsed_keys.push(Apostrophe),
                    '\\' | '|' => parsed_keys.push(BackSlash),
                    '[' | '{' => parsed_keys.push(LSquareBracket),
                    ']' | '}' => parsed_keys.push(RSquareBracket),
                    'a' | 'A' => parsed_keys.push(A),
                    'b' | 'B' => parsed_keys.push(B),
                    'c' | 'C' => parsed_keys.push(C),
                    'd' | 'D' => parsed_keys.push(D),
                    'e' | 'E' => parsed_keys.push(E),
                    'f' | 'F' => parsed_keys.push(F),
                    'g' | 'G' => parsed_keys.push(G),
                    'h' | 'H' => parsed_keys.push(H),
                    'i' | 'I' => parsed_keys.push(I),
                    'j' | 'J' => parsed_keys.push(J),
                    'k' | 'K' => parsed_keys.push(K),
                    'l' | 'L' => parsed_keys.push(L),
                    'm' | 'M' => parsed_keys.push(M),
                    'n' | 'N' => parsed_keys.push(N),
                    'o' | 'O' => parsed_keys.push(O),
                    'p' | 'P' => parsed_keys.push(P),
                    'q' | 'Q' => parsed_keys.push(Q),
                    'r' | 'R' => parsed_keys.push(R),
                    's' | 'S' => parsed_keys.push(S),
                    't' | 'T' => parsed_keys.push(T),
                    'u' | 'U' => parsed_keys.push(U),
                    'v' | 'V' => parsed_keys.push(V),
                    'w' | 'W' => parsed_keys.push(W),
                    'x' | 'X' => parsed_keys.push(X),
                    'y' | 'Y' => parsed_keys.push(Y),
                    'z' | 'Z' => parsed_keys.push(Z),
                    _ => parsed_keys.push(UK),
                }
            }
        }
        parsed_keys
    }
}
