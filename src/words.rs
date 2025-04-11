use std::{fs::read_to_string, io::Result, num::NonZeroUsize, path::Path};

use indexmap::{map::Iter, IndexMap};

use crate::chords::{Chord, Chords};

#[derive(Default)]
pub struct Entry {
    pub rank: Option<NonZeroUsize>,
    pub chord: Chord,
}

pub struct Words(IndexMap<String, Entry>);

impl Words {
    pub fn read_from_file_and_chords(path: impl AsRef<Path>, chords: &Chords) -> Result<Self> {
        let words = read_to_string(path)?;

        let mut entries: IndexMap<_, _> = words
            .split('\n')
            .enumerate()
            .map(|(i, word)| {
                (
                    word.to_owned(),
                    Entry {
                        rank: Some(NonZeroUsize::new(i + 1).unwrap()),
                        chord: Chord::default(),
                    },
                )
            })
            .collect();

        for (chord, word) in chords.iter() {
            entries.entry(word).or_default().chord = chord;
        }

        Ok(Self(entries))
    }

    pub fn iter(&self) -> Iter<String, Entry> {
        self.0.iter()
    }

    pub fn update_chord(&mut self, word: String, chord: Chord) {
        self.0.entry(word).or_default().chord = chord;
    }

    pub fn get_chord(&self, word: &String) -> Option<&Chord> {
        self.0.get(word).map(|entry| &entry.chord)
    }
}
