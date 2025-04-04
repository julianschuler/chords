use std::{
    char::ParseCharError,
    collections::{BTreeMap, BTreeSet},
    fmt::{Display, Formatter, Result as FmtResult},
    fs::{read_to_string, File},
    io::{Result as IoResult, Write},
    ops::Deref,
    path::Path,
    str::FromStr,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chord(BTreeSet<char>);

impl FromStr for Chord {
    type Err = ParseCharError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let chords: Result<_, _> = string
            .trim()
            .split('+')
            .map(|string| char::from_str(string).map(|char| char.to_ascii_uppercase()))
            .collect();

        Ok(Self(chords?))
    }
}

impl Display for Chord {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let chars: Vec<_> = self.0.iter().map(ToString::to_string).collect();

        write!(f, "{}", chars.join("+"))
    }
}

pub struct Chords(BTreeMap<Chord, String>);

impl Chords {
    pub fn read_from_file(path: impl AsRef<Path>) -> IoResult<Self> {
        let lines = read_to_string(path)?;

        let chords = lines
            .split('\n')
            .filter_map(|line| {
                let mut split = line.split(':');

                let chord: Chord = str::parse(split.next()?).ok()?;
                let word = split.next()?.trim().to_owned();

                Some((chord, word))
            })
            .collect();

        Ok(Self(chords))
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>) -> IoResult<()> {
        let lines: Vec<_> = self
            .0
            .iter()
            .map(|(chord, word)| format!("{chord}: {word}\n"))
            .collect();

        File::create(path)?.write_all(lines.concat().as_bytes())
    }
}

impl Deref for Chords {
    type Target = BTreeMap<Chord, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
