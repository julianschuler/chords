mod chords;
mod tui;

use std::io::Result;

use chords::Chords;
use tui::Tui;

fn main() -> Result<()> {
    const CHORDS_PATH: &str = "chords.txt";

    let mut tui = Tui::new()?;
    let chords = Chords::read_from_file(CHORDS_PATH)?;

    if let Err(error) = tui.run_event_loop() {
        eprintln!("Error when running event loop: {error}");
    }

    chords.write_to_file(CHORDS_PATH)?;
    tui.finish()?;

    Ok(())
}
