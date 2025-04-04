use std::io::{stdout, Result, Stdout};

use crossterm::{
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Span, Text},
    widgets::{Block, Paragraph, Row as TableRow, Table, TableState},
    Terminal,
};

use crate::{
    chords::{Chord, Chords},
    words::Words,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    words: Words,
    rows: Vec<Row>,
    error: String,
    search: String,
    table_state: TableState,
}

impl Tui {
    pub fn new(words: Words) -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            words,
            rows: Vec::new(),
            error: String::new(),
            search: String::new(),
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

    pub fn run_event_loop(&mut self, chords: &mut Chords) -> Result<()> {
        self.update_rows();

        loop {
            self.draw()?;

            let event = read()?;
            if let Event::Key(key) = event {
                if self.handle_key(key, chords) {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let layout =
                Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());

            let text = if !self.error.is_empty() {
                Span::from(self.error.as_str()).bold().red()
            } else {
                Span::from(self.search.as_str())
            };
            let block = Block::bordered().title(Span::from("Search chords").bold());

            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, layout[0]);

            let widths = [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ];
            let header = TableRow::new(["Rank", "Word", "Chord"]).bold();
            let block = Block::bordered();
            let table = Table::new(&self.rows, widths)
                .block(block)
                .header(header)
                .row_highlight_style(Style::new().reversed());
            frame.render_stateful_widget(table, layout[1], &mut self.table_state);
        })?;

        if self.table_state.selected().is_none() {
            let x: u16 = self.search.len().try_into().unwrap_or(u16::MAX - 1);
            self.terminal.set_cursor_position((x + 1, 1))?;
            self.terminal.show_cursor()?;
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, chords: &mut Chords) -> bool {
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
                            self.search.clear();
                            self.update_rows();
                        }
                        _ => {}
                    }
                } else {
                    match self.get_current_row() {
                        Some(row) => {
                            let word = row.word.clone();
                            let chord = &mut row.chord;

                            let chord_is_unique = self.error.is_empty();
                            if chord_is_unique {
                                // TODO: How to ensure that chord is only deleted if it is unique
                                chords.remove(chord);
                            }

                            if chord.insert(char) {
                                if let Some(conflicting_word) = chords.get_word(chord) {
                                    self.words.clear_chord(&word);
                                    self.error = format!(
                                        "'{word}' has the same chord as '{conflicting_word}'",
                                    );
                                } else {
                                    let chord = chord.clone();

                                    chords.insert(chord.clone(), word.clone());
                                    self.words.update_chord(word, chord);
                                    self.error.clear();
                                }
                            } else if !chord.is_empty() {
                                chords.insert(chord.clone(), word);
                            }
                        }
                        None => {
                            self.search.push(char);
                            self.update_rows();
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                match self.get_current_row() {
                    Some(row) => {
                        chords.remove(&row.chord);
                        row.chord.clear();

                        let word = &row.word.clone();
                        self.words.clear_chord(word);
                        self.error.clear();
                    }
                    None => {
                        self.search.pop();
                    }
                }
                self.update_rows();
            }
            KeyCode::Esc => self.unselect_row(),
            KeyCode::Up => self.select_previous_row(),
            KeyCode::Down => self.select_next_row(),
            KeyCode::Tab => self.select_next_row(),
            _ => {}
        }

        false
    }

    fn update_rows(&mut self) {
        self.rows = self
            .words
            .iter()
            .filter_map(|(word, entry)| {
                if !word.contains(&self.search) {
                    return None;
                }

                let word = word.to_owned();
                let rank = entry
                    .rank
                    .as_ref()
                    .map_or(String::new(), |rank| rank.to_string());
                let chord = entry.chord.clone();

                Some(Row { rank, word, chord })
            })
            .collect();
    }

    fn unselect_row(&mut self) {
        self.clear_chord_if_conflicting();

        self.table_state.select(None);
    }

    fn select_previous_row(&mut self) {
        self.clear_chord_if_conflicting();

        let row = self
            .table_state
            .selected()
            .and_then(|row| if row > 0 { Some(row - 1) } else { None });
        self.table_state.select(row);
    }

    fn select_next_row(&mut self) {
        self.clear_chord_if_conflicting();

        let row = self
            .table_state
            .selected()
            .map_or(Some(0), |row| Some(row + 1));
        self.table_state.select(row);
    }

    fn clear_chord_if_conflicting(&mut self) {
        if !self.error.is_empty() {
            self.update_rows();
            self.error.clear();
        }
    }

    fn get_current_row(&mut self) -> Option<&mut Row> {
        self.table_state
            .selected()
            .and_then(|index| self.rows.get_mut(index))
    }
}

struct Row {
    rank: String,
    word: String,
    chord: Chord,
}

impl<'a> From<&'a Row> for TableRow<'a> {
    fn from(row: &'a Row) -> Self {
        TableRow::new([row.rank.as_str(), row.word.as_str(), row.chord.as_str()])
    }
}
