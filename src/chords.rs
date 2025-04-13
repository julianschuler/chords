use std::{
    char::ParseCharError,
    collections::{btree_map::IntoIter, BTreeMap, BTreeSet},
    fs::{read_to_string, File},
    io::{Result as IoResult, Write},
    path::Path,
    str::FromStr,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chord(String);

impl Chord {
    pub fn insert(&mut self, key: char) -> bool {
        let key = key.to_ascii_uppercase();
        if !key.is_ascii_uppercase() || self.0.contains(key) {
            return false;
        }

        if self.0.is_empty() {
            self.0 = String::from(key)
        } else {
            match self.0.find(|char| char != '+' && key < char) {
                Some(position) => {
                    self.0.insert_str(position, &format!("{key}+"));
                }
                None => {
                    self.0.push('+');
                    self.0.push(key);
                }
            }
        }

        true
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn keys(&self) -> impl Iterator<Item = char> + use<'_> {
        self.0.chars().filter(|&char| char != '+')
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for Chord {
    type Err = ParseCharError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let chords: Result<BTreeSet<_>, _> = string
            .split('+')
            .map(|string| char::from_str(string.trim()).map(|char| char.to_ascii_uppercase()))
            .collect();

        let chars: Vec<_> = chords?.iter().map(ToString::to_string).collect();

        Ok(Self(chars.join("+")))
    }
}

pub struct Chords<'a> {
    chords: BTreeMap<Chord, String>,
    chords_path: &'a Path,
    export_path: &'a Path,
}

impl<'a> Chords<'a> {
    pub fn new(chords_path: &'a Path, export_path: &'a Path) -> IoResult<Self> {
        let lines = read_to_string(chords_path)?;

        let chords = lines
            .split('\n')
            .filter_map(|line| {
                let mut split = line.split(':');

                let chord: Chord = split.next()?.parse().ok()?;
                let word = split.next()?.trim().to_owned();

                Some((chord, word))
            })
            .collect();

        Ok(Self {
            chords,
            chords_path,
            export_path,
        })
    }

    pub fn iter(&self) -> IntoIter<Chord, String> {
        self.chords.clone().into_iter()
    }

    pub fn remove(&mut self, chord: &Chord) -> Option<String> {
        self.chords.remove(chord)
    }

    pub fn insert(&mut self, chord: Chord, word: String) -> Option<String> {
        self.chords.insert(chord, word)
    }

    pub fn get_word(&self, chord: &Chord) -> Option<&String> {
        self.chords.get(chord)
    }

    pub fn save_and_export(&self) -> IoResult<()> {
        self.save()?;
        self.export()?;

        Ok(())
    }

    fn save(&self) -> IoResult<()> {
        let lines: Vec<_> = self
            .chords
            .iter()
            .map(|(chord, word)| format!("{chord}: {word}\n", chord = chord.as_str()))
            .collect();

        File::create(self.chords_path)?.write_all(lines.concat().as_bytes())
    }

    fn export(&self) -> IoResult<()> {
        const PREFIX: &str = "DE_";

        let lines: Vec<_> = self
            .chords
            .iter()
            .map(|(chord, word)| {
                let keys: Vec<_> = chord.keys().map(|key| format!("{PREFIX}{key}")).collect();
                let keys = keys.join(", ");
                format!("SUBS(_{word}, \"{word}\", {keys})\n")
            })
            .collect();

        File::create(self.export_path)?.write_all(lines.concat().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chords() {
        let chord1: Chord = "  B + a+c ".parse().unwrap();
        let chord2: Chord = "c+B+a".parse().unwrap();
        let invalid: Result<Chord, _> = "aa+b".parse();

        assert_eq!(chord1, chord2);
        assert_eq!(chord1.as_str(), "A+B+C");
        assert!(invalid.is_err());
    }

    #[test]
    fn insert_into_chords() {
        let chord: Chord = "B+D".parse().unwrap();
        assert_eq!(chord.as_str(), "B+D");

        let mut insert_invalid = chord.clone();
        assert!(!insert_invalid.insert('/'));
        assert_eq!(insert_invalid, chord);

        let mut insert_contained = chord.clone();
        assert!(!insert_contained.insert('b'));
        assert_eq!(insert_invalid, chord);

        let mut insert_front = chord.clone();
        assert!(insert_front.insert('a'));
        assert_eq!(insert_front.as_str(), "A+B+D");

        let mut insert_center = chord.clone();
        assert!(insert_center.insert('c'));
        assert_eq!(insert_center.as_str(), "B+C+D");

        let mut insert_end = chord.clone();
        assert!(insert_end.insert('e'));
        assert_eq!(insert_end.as_str(), "B+D+E");

        let mut insert_empty = Chord::default();
        assert_eq!(insert_empty.as_str(), "");
        assert!(insert_empty.insert('d'));
        assert_eq!(insert_empty.as_str(), "D");
    }
}
