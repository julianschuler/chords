use std::io::{stdout, Result};

use chords::Chords;
use crossterm::{
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

mod chords;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let chords_path = "chords.txt";
    let chords = Chords::read_from_file(chords_path)?;

    if let Err(error) = run_event_loop(&mut terminal) {
        eprintln!("Error when running main loop: {error}");
    }

    chords.write_to_file(chords_path)?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}

fn run_event_loop<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut search_field = String::new();

    loop {
        draw_tui(terminal, &search_field)?;

        let event = read()?;
        if let Event::Key(key) = event {
            if handle_key(key, &mut search_field) {
                break;
            }
        }
    }

    Ok(())
}

fn draw_tui<B: Backend>(terminal: &mut Terminal<B>, search_field: &str) -> Result<()> {
    terminal.draw(|frame| {
        let text = Text::from(search_field);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Search chords");
        let paragraph = Paragraph::new(text).block(block);

        frame.render_widget(paragraph, frame.size());
    })?;

    let x: u16 = search_field.len().try_into().unwrap();
    terminal.set_cursor(x + 1, 1)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key(key: KeyEvent, search_field: &mut String) -> bool {
    match key.code {
        KeyCode::Char(char) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('c') => {
                        // ctrl-c
                        return true;
                    }
                    KeyCode::Char('h') => {
                        // ctrl-backspace
                        search_field.clear();
                    }
                    _ => {}
                }
            } else {
                search_field.push(char);
            }
        }
        KeyCode::Backspace => {
            search_field.pop();
        }
        _ => {}
    }

    false
}
