use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyChord {
    pub modifiers: Vec<Key>,
    pub key: Key,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Key {
    pub name: &'static str,
    pub code: u16,
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyParseError {
    name: String,
}

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported key name '{}'", self.name)
    }
}

impl std::error::Error for KeyParseError {}

impl FromStr for KeyChord {
    type Err = KeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut keys = s.split('+').map(str::trim).filter(|part| !part.is_empty());
        let mut parts: Vec<Key> = keys
            .by_ref()
            .map(parse_key)
            .collect::<Result<Vec<_>, _>>()?;

        let key = parts.pop().ok_or_else(|| KeyParseError {
            name: s.to_string(),
        })?;

        Ok(Self {
            modifiers: parts,
            key,
        })
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            write!(f, "{}+", modifier.name)?;
        }

        write!(f, "{}", self.key.name)
    }
}

pub fn parse_key(name: &str) -> Result<Key, KeyParseError> {
    let normalized = name.to_ascii_lowercase().replace(['_', '-'], "");

    let key = match normalized.as_str() {
        "esc" | "escape" => key("esc", 1),
        "1" => key("1", 2),
        "2" => key("2", 3),
        "3" => key("3", 4),
        "4" => key("4", 5),
        "5" => key("5", 6),
        "6" => key("6", 7),
        "7" => key("7", 8),
        "8" => key("8", 9),
        "9" => key("9", 10),
        "0" => key("0", 11),
        "a" => key("a", 30),
        "c" => key("c", 46),
        "g" => key("g", 34),
        "i" => key("i", 23),
        "j" => key("j", 36),
        "k" => key("k", 37),
        "l" => key("l", 38),
        "o" => key("o", 24),
        "r" => key("r", 19),
        "s" => key("s", 31),
        "t" => key("t", 20),
        "v" => key("v", 47),
        "x" => key("x", 45),
        "z" => key("z", 44),
        "space" => key("space", 57),
        "delete" | "del" => key("delete", 111),
        "left" => key("left", 105),
        "right" => key("right", 106),
        "up" => key("up", 103),
        "down" => key("down", 108),
        "home" => key("home", 102),
        "end" => key("end", 107),
        "ctrl" | "control" | "leftctrl" => key("leftctrl", 29),
        "shift" | "leftshift" => key("leftshift", 42),
        "alt" | "leftalt" => key("leftalt", 56),
        "enter" | "return" => key("enter", 28),
        _ => {
            return Err(KeyParseError {
                name: name.to_string(),
            })
        }
    };

    Ok(key)
}

pub fn all_keys(chords: &[KeyChord]) -> Vec<Key> {
    let mut keys = Vec::new();

    for chord in chords {
        keys.extend(chord.modifiers.iter().copied());
        keys.push(chord.key);
    }

    keys.sort();
    keys.dedup();
    keys
}

fn key(name: &'static str, code: u16) -> Key {
    Key { name, code }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_key() {
        assert_eq!("space".parse::<KeyChord>().unwrap().key.code, 57);
    }

    #[test]
    fn parses_chord() {
        let chord = "ctrl+shift+z".parse::<KeyChord>().unwrap();

        assert_eq!(chord.modifiers.len(), 2);
        assert_eq!(chord.key.name, "z");
    }

    #[test]
    fn rejects_unknown_key() {
        assert!("hyper".parse::<KeyChord>().is_err());
    }
}
