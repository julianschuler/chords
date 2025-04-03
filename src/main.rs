use std::io::{stdout, Result};

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

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut search_field = String::new();

    loop {
        draw_tui(&mut terminal, &search_field)?;

        let event = read()?;
        if let Event::Key(key) = event {
            if handle_key(key, &mut search_field) {
                break;
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

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
