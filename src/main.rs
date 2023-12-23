mod supermemo;
mod theme;

use std::{error::Error, io};

use crate::theme::THEME;
/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * An input box always focused. Every character you type is registered
///   here.
///   * An entered character is inserted at the cursor position.
///   * Pressing Backspace erases the left character before the cursor position
///   * Pressing Enter pushes the current input in the history of previous
///   messages.
/// **Note: ** as this is a relatively simple example unicode characters are unsupported and
/// their use will result in undefined behaviour.
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use supermemo::Deck;

#[derive(Clone, Copy)]
enum AnswerStatus {
    Show,
    Hide,
}

impl AnswerStatus {
    fn flip(self) -> Self {
        match self {
            AnswerStatus::Show => AnswerStatus::Hide,
            AnswerStatus::Hide => AnswerStatus::Show,
        }
    }
}

/// App holds the state of the application
struct App {
    question: String,
    answer: String,
    answer_status: AnswerStatus,
}

impl App {
    fn toggle(&mut self) {
        self.answer_status = self.answer_status.flip();
    }

    fn get_answer(&self) -> &str {
        match self.answer_status {
            AnswerStatus::Show => &self.answer,
            AnswerStatus::Hide => "",
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut deck = Deck::fake_data();
    let Some(mut app) = next(&deck) else {
        return Ok(());
    };

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match &app.answer_status {
                AnswerStatus::Show => match key.code {
                    KeyCode::Char('h') => {
                        let Some(new_app) = next(&deck) else {
                            return Ok(());
                        };
                        app = new_app
                    }
                    KeyCode::Char('g') => {
                        let Some(new_app) = next(&deck) else {
                            return Ok(());
                        };
                        app = new_app
                    }
                    KeyCode::Char('f') => {
                        let Some(new_app) = next(&deck) else {
                            return Ok(());
                        };
                        app = new_app
                    }
                    KeyCode::Char(' ') => app.toggle(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                AnswerStatus::Hide => match key.code {
                    KeyCode::Char(' ') => app.toggle(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                }, /*
                   InputMode::Normal => match key.code {
                       KeyCode::Char('e') => {
                           app.input_mode = InputMode::Editing;
                       }
                       KeyCode::Char('q') => {
                           return Ok(());
                       }
                       _ => {}
                   },
                   InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                       KeyCode::Enter => app.submit_message(),
                       KeyCode::Char(to_insert) => {
                           app.enter_char(to_insert);
                       }
                       KeyCode::Backspace => {
                           app.delete_char();
                       }
                       KeyCode::Left => {
                           app.move_cursor_left();
                       }
                       KeyCode::Right => {
                           app.move_cursor_right();
                       }
                       KeyCode::Esc => {
                           app.input_mode = InputMode::Normal;
                       }
                       _ => {}
                   },
                   _ => {}
                    */
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // question
            Constraint::Min(1),    // answer
            Constraint::Length(1), // button
        ])
        .split(f.size());

    let question = Paragraph::new(app.question.as_str())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(question, chunks[0]);

    let answer = Paragraph::new(app.get_answer())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(answer, chunks[1]);

    let escape_keys = [("Q/Esc", "Quit")];
    let hide_keys = [("<Space>", "Show answer")];
    let show_keys = [("f", "Forget"), ("h", "Hard"), ("g", "Good")];

    let keys: &[(&str, &str)] = match app.answer_status {
        AnswerStatus::Show => &show_keys,
        AnswerStatus::Hide => &hide_keys,
    };

    let spans = escape_keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {} ", key), THEME.key_binding.key);
            let desc = Span::styled(format!(" {} ", desc), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();
    let buttons = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Right)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232));
    f.render_widget(buttons, chunks[2]);

    let spans = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {} ", key), THEME.key_binding.key);
            let desc = Span::styled(format!(" {} ", desc), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();
    let buttons = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236));
    f.render_widget(buttons, chunks[2]);
}

fn next(deck: &Deck) -> Option<App> {
    let Some(question) = deck.search_reviewable() else {
        return None;
    };
    let answer = ghost_get_answer(&question);

    Some(App {
        question,
        answer,
        answer_status: AnswerStatus::Hide,
    })
}

fn ghost_get_answer(_question: &str) -> String {
    "this is answer".to_owned()
}
