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
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Span, Text, ToText},
    widgets::{Block, Paragraph, Row, Table, TableState},
    Terminal,
};

use crate::words::Words;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    search_field: String,
    table_state: TableState,
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
            table_state: TableState::new(),
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

    pub fn run_event_loop(&mut self, words: Words) -> Result<()> {
        loop {
            self.draw(&words)?;

            let event = read()?;
            if let Event::Key(key) = event {
                if self.handle_key(key) {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, words: &Words) -> Result<()> {
        self.terminal.draw(|frame| {
            let layout =
                Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());

            let text = Text::from(self.search_field.as_str());
            let block =
                Block::bordered().title(Span::from("Search chords").style(Style::new().bold()));
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, layout[0]);

            let widths = [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ];
            let rows = words.iter().map(|(word, entry)| {
                let word = word.to_text();
                let rank = entry
                    .rank
                    .as_ref()
                    .map_or(Text::default(), |rank| rank.to_text());
                let chord = entry
                    .chord
                    .as_ref()
                    .map_or(Text::default(), |chord| chord.to_text());

                Row::new(vec![rank, word, chord])
            });
            let header = Row::new(vec!["Rank", "Word", "Chord"]).style(Style::new().bold());
            let block = Block::bordered();
            let table = Table::new(rows, widths)
                .block(block)
                .header(header)
                .row_highlight_style(Style::new().reversed());
            frame.render_stateful_widget(table, layout[1], &mut self.table_state);
        })?;

        let x: u16 = self.search_field.len().try_into().unwrap_or(u16::MAX - 1);
        self.terminal.set_cursor_position((x + 1, 1))?;
        self.terminal.show_cursor()?;

        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

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
            KeyCode::Up => self.select_previous_row(),
            KeyCode::Down => self.select_next_row(),
            _ => {}
        }

        false
    }

    pub fn select_previous_row(&mut self) {
        let row = self
            .table_state
            .selected()
            .and_then(|row| if row > 0 { Some(row - 1) } else { None });
        self.table_state.select(row);
    }

    pub fn select_next_row(&mut self) {
        let row = self
            .table_state
            .selected()
            .map_or(Some(0), |row| Some(row + 1));
        self.table_state.select(row);
    }
}
