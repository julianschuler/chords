use std::{fs::read_to_string, io::Result, num::NonZeroUsize, ops::Deref, path::Path};

use indexmap::IndexMap;

use crate::chords::{Chord, Chords};

#[derive(Default)]
pub struct Entry {
    pub rank: Option<NonZeroUsize>,
    pub chord: Option<Chord>,
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
                        chord: None,
                    },
                )
            })
            .collect();

        for (chord, word) in chords.iter() {
            entries.entry(word.clone()).or_default().chord = Some(chord.clone());
        }

        Ok(Self(entries))
    }
}

impl Deref for Words {
    type Target = IndexMap<String, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
