use std::io::{stdout, Result, Stdout};

use crossterm::{
    event::{
        read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Span,
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
    search: String,
    table_state: TableState,
}

impl Tui {
    pub fn new(words: Words) -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            words,
            rows: Vec::new(),
            search: String::new(),
            table_state: TableState::new(),
        })
    }

    pub fn finish(mut self, chords: &mut Chords) -> Result<()> {
        self.commit_changes(chords)?;

        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            PopKeyboardEnhancementFlags
        )?;

        Ok(())
    }

    pub fn run_event_loop(&mut self, chords: &mut Chords) -> Result<()> {
        self.update_rows();

        loop {
            self.draw(chords)?;

            let event = read()?;
            if let Event::Key(key) = event {
                if self.handle_key(key, chords)? {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, chords: &Chords) -> Result<()> {
        let error_message = self.error_message(chords);

        self.terminal.draw(|frame| {
            let layout =
                Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());

            let text = match error_message {
                Some(error) => Span::from(error).bold().red(),
                None => Span::from(self.search.as_str()),
            };
            let block =
                Block::bordered().title(Span::from("Search chords").style(Style::new().bold()));
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

    fn error_message(&mut self, chords: &Chords) -> Option<String> {
        if let Some(row) = self.current_row_mut() {
            let word = &row.word;

            if let Some(stored_word) = chords.get_word(&row.chord) {
                if stored_word != word {
                    let error = format!("'{word}' has the same chord as '{stored_word}'",);
                    return Some(error);
                }
            }
        }

        None
    }

    fn handle_key(&mut self, key: KeyEvent, chords: &mut Chords) -> Result<bool> {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                KeyCode::Char('c') => {
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    self.search.clear();
                    self.update_rows();
                }
                _ => {}
            }

            return Ok(false);
        }

        match key.code {
            KeyCode::Char(char) => match self.current_row_mut() {
                Some(row) => {
                    row.chord.insert(char);
                }
                None => {
                    self.search.push(char);
                    self.update_rows();
                }
            },
            KeyCode::Backspace => match self.current_row_mut() {
                Some(row) => {
                    row.chord.clear();
                }
                None => {
                    self.search.pop();
                    self.update_rows();
                }
            },
            KeyCode::Esc => self.unselect_row(chords)?,
            KeyCode::Up => self.select_previous_row(chords)?,
            KeyCode::Down => self.select_next_row(chords)?,
            KeyCode::Tab => self.select_next_row(chords)?,
            _ => {}
        }

        Ok(false)
    }

    fn current_row(&self) -> Option<&Row> {
        self.table_state
            .selected()
            .and_then(|index| self.rows.get(index))
    }

    fn current_row_mut(&mut self) -> Option<&mut Row> {
        self.table_state
            .selected()
            .and_then(|index| self.rows.get_mut(index))
    }

    fn reset_current_row(&mut self) {
        if let Some(row) = self
            .table_state
            .selected()
            .and_then(|index| self.rows.get_mut(index))
        {
            row.chord = self.words.get_chord(&row.word).cloned().unwrap_or_default();
        }
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

    fn unselect_row(&mut self, chords: &mut Chords) -> Result<()> {
        self.commit_changes(chords)?;

        self.table_state.select(None);

        Ok(())
    }

    fn select_previous_row(&mut self, chords: &mut Chords) -> Result<()> {
        self.commit_changes(chords)?;

        let row = self
            .table_state
            .selected()
            .and_then(|row| if row > 0 { Some(row - 1) } else { None });
        self.table_state.select(row);

        Ok(())
    }

    fn select_next_row(&mut self, chords: &mut Chords) -> Result<()> {
        self.commit_changes(chords)?;

        let row = self
            .table_state
            .selected()
            .map_or(Some(0), |row| Some(row + 1));
        self.table_state.select(row);

        Ok(())
    }

    fn commit_changes(&mut self, chords: &mut Chords) -> Result<()> {
        if let Some(row) = self.current_row() {
            let word = row.word.clone();
            let chord = row.chord.clone();

            if chords.get_word(&chord).is_none() {
                if let Some(old_chord) = self.words.get_chord(&word) {
                    if !old_chord.is_empty() {
                        chords.remove(old_chord);
                    }
                }

                self.words.update_chord(word.clone(), chord.clone());
                if !chord.is_empty() {
                    chords.insert(chord, word);
                }
                chords.save_and_export()?;
            } else {
                self.reset_current_row();
            }
        }

        Ok(())
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
