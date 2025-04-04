use std::io::{stdout, Result, Stdout};

use crossterm::{
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    search_field: String,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            search_field: String::new(),
        })
    }

    pub fn finish(mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        Ok(())
    }

    pub fn run_event_loop(&mut self) -> Result<()> {
        loop {
            self.draw()?;

            let event = read()?;
            if let Event::Key(key) = event {
                if self.handle_key(key) {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let text = Text::from(self.search_field.as_str());
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Search chords");
            let paragraph = Paragraph::new(text).block(block);

            frame.render_widget(paragraph, frame.size());
        })?;

        let x: u16 = self.search_field.len().try_into().unwrap_or(u16::MAX - 1);
        self.terminal.set_cursor(x + 1, 1)?;
        self.terminal.show_cursor()?;

        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
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
                            self.search_field.clear();
                        }
                        _ => {}
                    }
                } else {
                    self.search_field.push(char);
                }
            }
            KeyCode::Backspace => {
                self.search_field.pop();
            }
            _ => {}
        }

        false
    }
}
