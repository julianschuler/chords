mod chords;
mod tui;
mod words;

use std::io::Result;

use chords::Chords;
use tui::Tui;
use words::Words;

fn main() -> Result<()> {
    const CHORDS_PATH: &str = "chords.txt";
    const WORDS_PATH: &str = "words.txt";

    let mut tui = Tui::new()?;
    let chords = Chords::read_from_file(CHORDS_PATH)?;
    let words = Words::read_from_file_and_chords(WORDS_PATH, &chords)?;

    if let Err(error) = tui.run_event_loop() {
        eprintln!("Error when running event loop: {error}");
    }

    chords.write_to_file(CHORDS_PATH)?;
    tui.finish()?;

    Ok(())
}
