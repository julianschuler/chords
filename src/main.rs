use std::io::{stdout, Result};

use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
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
        terminal.draw(|frame| {
            let text = Text::from(search_field.as_str());
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Search chords");
            let paragraph = Paragraph::new(text).block(block);

            frame.render_widget(paragraph, frame.size());
        })?;
        terminal.show_cursor()?;

        let x: u16 = search_field.len().try_into().unwrap();
        terminal.set_cursor(x + 1, 1)?;

        let event = read()?;

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(char) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('c') => {
                                // ctrl-c
                                break;
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
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
