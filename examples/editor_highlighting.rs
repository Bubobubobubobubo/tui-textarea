use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{terminal, ExecutableCommand};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::borrow::Cow;
use std::env;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use tui_textarea::{CursorMove, Input, Key, TextArea, SyntaxHighlighter};

macro_rules! error {
    ($fmt: expr $(, $args:tt)*) => {{
        Err(io::Error::new(io::ErrorKind::Other, format!($fmt $(, $args)*)))
    }};
}

// Sample text for different languages
const TEXT_RUST: &str = r#"
use std::io;

fn main() -> io::Result<()> {
    println!("Hello, world!");
    Ok(())
}
"#;

const TEXT_PYTHON: &str = r#"
import sys

def main():
    print("Hello, world!")
    return 0

if __name__ == "__main__":
    sys.exit(main())
"#;

const TEXT_CPP: &str = r#"
#include <iostream>

int main() {
    std::cout << "Hello, world!" << std::endl;
    return 0;
}
"#;

const TEXT_LISP: &str = r#"
(defun hello ()
  (print "Hello, world!"))

(hello)
"#;

const TEXT_JAVASCRIPT: &str = r#"
function main() {
    console.log("Hello, world!");
    return 0;
}

main();
"#;

struct SearchBox<'a> {
    textarea: TextArea<'a>,
    open: bool,
}

impl Default for SearchBox<'_> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self {
            textarea,
            open: false,
        }
    }
}

impl SearchBox<'_> {
    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        // Remove input for next search. Do not recreate `self.textarea` instance to keep undo history so that users can
        // restore previous input easily.
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.delete_line_by_head();
    }

    fn height(&self) -> u16 {
        if self.open {
            3
        } else {
            0
        }
    }

    fn input(&mut self, input: Input) -> Option<&'_ str> {
        match input {
            Input {
                key: Key::Enter, ..
            }
            | Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            } => None, // Disable shortcuts which inserts a newline. See `single_line` example
            input => {
                let modified = self.textarea.input(input);
                modified.then(|| self.textarea.lines()[0].as_str())
            }
        }
    }

    fn set_error(&mut self, err: Option<impl Display>) {
        let b = if let Some(err) = err {
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Search: {}", err))
                .style(Style::default().fg(Color::Red))
        } else {
            Block::default().borders(Borders::ALL).title("Search")
        };
        self.textarea.set_block(b);
    }
}

struct Buffer<'a> {
    textarea: TextArea<'a>,
    path: PathBuf,
    modified: bool,
}

impl Buffer<'_> {
    fn new(path: PathBuf) -> io::Result<Self> {
        let mut textarea = if let Ok(md) = path.metadata() {
            if md.is_file() {
                let mut textarea: TextArea = io::BufReader::new(fs::File::open(&path)?)
                    .lines()
                    .collect::<io::Result<_>>()?;
                if textarea.lines().iter().any(|l| l.starts_with('\t')) {
                    textarea.set_hard_tab_indent(true);
                }
                textarea
            } else {
                return error!("{:?} is not a file", path);
            }
        } else {
            TextArea::default() // File does not exist
        };
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        Ok(Self {
            textarea,
            path,
            modified: false,
        })
    }

    fn save(&mut self) -> io::Result<()> {
        if !self.modified {
            return Ok(());
        }
        let mut f = io::BufWriter::new(fs::File::create(&self.path)?);
        for line in self.textarea.lines() {
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
        }
        self.modified = false;
        Ok(())
    }
}

struct State<'a> {
    textarea: TextArea<'a>,
    highlighter: SyntaxHighlighter,
    syntaxes: Vec<(&'static str, &'static str)>, // (Name, Content)
    current_syntax_idx: usize,
    theme_name: String,
    message: Option<Cow<'static, str>>,
}

impl<'a> State<'a> {
    fn new() -> Self {
        let highlighter = SyntaxHighlighter::new();
        let syntaxes = vec![
            ("Rust", TEXT_RUST),
            ("Python", TEXT_PYTHON),
            ("C++", TEXT_CPP),
            ("Lisp", TEXT_LISP),
            ("JavaScript", TEXT_JAVASCRIPT),
        ];
        let current_syntax_idx = 0;
        let theme_name = "base16-ocean.dark".to_string(); // Default theme

        let mut textarea = TextArea::from(syntaxes[current_syntax_idx].1.lines());
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Syntax Highlighting Editor"),
        );
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        textarea.set_syntax_highlighter(highlighter.clone()); // Clone the Arc-wrapped sets
        textarea.set_syntax(Some(syntaxes[current_syntax_idx].0.to_string()));
        textarea.set_theme(Some(theme_name.clone()));

        State {
            textarea,
            highlighter,
            syntaxes,
            current_syntax_idx,
            theme_name,
            message: None,
        }
    }

    fn next_syntax(&mut self) {
        self.current_syntax_idx = (self.current_syntax_idx + 1) % self.syntaxes.len();
        self.update_textarea_content_and_syntax();
        self.message = Some(Cow::Borrowed("Changed syntax"));
    }

    fn prev_syntax(&mut self) {
         self.current_syntax_idx = if self.current_syntax_idx == 0 {
            self.syntaxes.len() - 1
        } else {
            self.current_syntax_idx - 1
        };
        self.update_textarea_content_and_syntax();
        self.message = Some(Cow::Borrowed("Changed syntax"));
    }

    fn update_textarea_content_and_syntax(&mut self) {
        let (name, content) = self.syntaxes[self.current_syntax_idx];
        // Replace content - create a new TextArea to easily replace lines
        let mut new_textarea = TextArea::from(content.lines());
        new_textarea.set_block(self.textarea.block().cloned().unwrap_or_default()); // Keep block
        new_textarea.set_line_number_style(self.textarea.line_number_style().unwrap_or_default()); // Keep line numbers
        new_textarea.set_syntax_highlighter(self.highlighter.clone());
        new_textarea.set_syntax(Some(name.to_string()));
        new_textarea.set_theme(Some(self.theme_name.clone()));
        // Try to preserve cursor position roughly
        let (r, c) = self.textarea.cursor();
        let new_max_row = new_textarea.lines().len().saturating_sub(1);
        let new_max_col = new_textarea.lines().get(r.min(new_max_row)).map_or(0, |l| l.chars().count());
        new_textarea.move_cursor(tui_textarea::CursorMove::Jump(r.min(new_max_row) as u16, c.min(new_max_col) as u16));
        self.textarea = new_textarea; // Replace the old textarea
    }

    fn current_language_name(&self) -> &'static str {
        self.syntaxes[self.current_syntax_idx].0
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut state = State::new();

    loop {
        term.draw(|f| ui(f, &state))?;
        if let Event::Key(key) = crossterm::event::read()? {
             // Clear message on next input
             state.message = None;
             match key {
                // Language Switching
                KeyEvent {
                    code: KeyCode::Char('n'),
                    modifiers: KeyModifiers::CONTROL, ..
                } => state.next_syntax(),
                 KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::CONTROL, ..
                } => state.prev_syntax(),
                // Quit
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL, ..
                } => break,
                // Other inputs for the textarea
                _ => {
                    let input = Input::from(key);
                    state.textarea.input(input);
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    term.backend_mut().execute(terminal::LeaveAlternateScreen)?;
    term.show_cursor()?;
    Ok(())
}

fn ui(f: &mut ratatui::Frame, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        // Borrow the result of the if/else expression
        .constraints(&(if state.message.is_some() {
            vec![Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)]
        } else {
            vec![Constraint::Min(1), Constraint::Length(1)]
        }))
        .split(f.area()); // Use area() instead of size()

    f.render_widget(&state.textarea, chunks[0]); // Use reference directly

    let status_text = format!(
        "Language: {} | Theme: {} | Ctrl+N: Next | Ctrl+P: Prev | Ctrl+Q: Quit",
        state.current_language_name(),
        state.theme_name
    );
    let status_bar = Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray));
    f.render_widget(status_bar, chunks[chunks.len()-1]); // Status bar always last

    // Render message if present
    if let Some(msg) = &state.message {
        let msg_p = Paragraph::new(msg.as_ref()).style(Style::default().fg(Color::Yellow));
        f.render_widget(msg_p, chunks[1]); // Message goes between editor and status
    }
}
