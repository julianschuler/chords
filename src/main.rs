mod chords;
mod tui;
mod words;

use std::{io::Result, path::PathBuf};

use chords::Chords;
use clap::Parser;
use tui::Tui;
use words::Words;

/// Small TUI program to manage a library of chords for QMK
#[derive(Parser)]
struct Args {
    /// Path of the file to read/write all chords from/to
    #[arg(short, long)]
    chords: PathBuf,
    /// Path to a list of common words sorted by frequeny in descending order
    #[arg(short, long)]
    words: PathBuf,
    /// Path to the file to export the combos to for use with QMK
    #[arg(short, long)]
    export: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut chords = Chords::read_from_file(&args.chords)?;
    chords.export(&args.export)?;
    let words = Words::read_from_file_and_chords(args.words, &chords)?;
    let mut tui = Tui::new(words)?;

    if let Err(error) = tui.run_event_loop(&mut chords) {
        eprintln!("Error when running event loop: {error}");
    }

    chords.write_to_file(args.chords)?;
    chords.export(args.export)?;
    tui.finish()?;

    Ok(())
}
