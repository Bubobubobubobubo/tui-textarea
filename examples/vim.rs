use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Operator(char),
}

impl Mode {
    fn block<'a>(&self) -> Block<'a> {
        let help = match self {
            Self::Normal => "type q to quit, type i to enter insert mode",
            Self::Insert => "type Esc to back to normal mode",
            Self::Visual => "type y to yank, type d to delete, type Esc to back to normal mode",
            Self::Operator(_) => "move cursor to apply operator",
        };
        let title = format!("{} MODE ({})", self, help);
        Block::default().borders(Borders::ALL).title(title)
    }

    fn cursor_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Operator(_) => Color::LightGreen,
        };
        Style::default().fg(color).add_modifier(Modifier::REVERSED)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
            Self::Visual => write!(f, "VISUAL"),
            Self::Operator(c) => write!(f, "OPERATOR({})", c),
        }
    }
}

// How the Vim emulation state transitions
enum Transition {
    Nop,
    Mode(Mode),
    Pending(Input),
    Quit,
}

// State of Vim emulation
struct Vim {
    mode: Mode,
    pending: Input, // Pending input to handle a sequence with two keys like gg
}

impl Vim {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            pending: Input::default(),
        }
    }

    fn with_pending(self, pending: Input) -> Self {
        Self {
            mode: self.mode,
            pending,
        }
    }

    fn transition(&self, input: Input, textarea: &mut TextArea<'_>) -> Transition {
        if input.key == Key::Null {
            return Transition::Nop;
        }

        match self.mode {
            Mode::Normal | Mode::Visual | Mode::Operator(_) => {
                match input {
                    Input {
                        key: Key::Char('h'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Back);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('j'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Down);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('k'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Up);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('l'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Forward);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('w'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordForward);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('e'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordEnd);
                        if matches!(self.mode, Mode::Operator(_)) {
                            textarea.move_cursor(CursorMove::Forward); // Include the text under the cursor
                        }
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordBack);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('^'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Head);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('$'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::End);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('D'),
                        ..
                    } => {
                        textarea.delete_line_by_end();
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('C'),
                        ..
                    } => {
                        textarea.delete_line_by_end();
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('p'),
                        ..
                    } => {
                        vim_paste_below(textarea);
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('P'),
                        ..
                    } => {
                        vim_paste_above(textarea);
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.undo();
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('r'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.redo();
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('x'),
                        ..
                    } => {
                        textarea.delete_next_char();
                        constrain_cursor_for_normal_mode(textarea);
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('i'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('a'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Forward);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('A'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('o'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::End);
                        textarea.insert_newline();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('O'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.insert_newline();
                        textarea.move_cursor(CursorMove::Up);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('I'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Head);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('q'),
                        ..
                    } => return Transition::Quit,
                    Input {
                        key: Key::Char('e'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll((1, 0));
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll((-1, 0));
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::HalfPageDown);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::HalfPageUp);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('f'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::PageDown);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('b'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::PageUp);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.start_selection();
                        return Transition::Mode(Mode::Visual);
                    }
                    Input {
                        key: Key::Char('V'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Visual);
                    }
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        ..
                    } if matches!(
                        self.pending,
                        Input {
                            key: Key::Char('g'),
                            ctrl: false,
                            ..
                        }
                    ) =>
                    {
                        textarea.move_cursor(CursorMove::Top);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char('G'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Bottom);
                        constrain_cursor_for_normal_mode(textarea);
                    }
                    Input {
                        key: Key::Char(c),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Operator(c) => {
                        // Handle yy, dd, cc. Select the entire line
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        let cursor = textarea.cursor();
                        textarea.move_cursor(CursorMove::Down);
                        if cursor == textarea.cursor() {
                            // At the last line, select to end of line
                            textarea.move_cursor(CursorMove::End);
                        } else {
                            // Move to beginning of next line to include the newline
                            textarea.move_cursor(CursorMove::Head);
                        }
                    }
                    Input {
                        key: Key::Char(op @ ('y' | 'd' | 'c')),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.start_selection();
                        return Transition::Mode(Mode::Operator(op));
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        // In Vim, visual selection is inclusive, but TextArea selection is already inclusive
                        // Don't move cursor forward as it adds extra characters
                        textarea.copy();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        // In Vim, visual selection is inclusive, but TextArea selection is already inclusive
                        // Don't move cursor forward as it adds extra characters
                        textarea.cut();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('c'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        // In Vim, visual selection is inclusive, but TextArea selection is already inclusive
                        // Don't move cursor forward as it adds extra characters
                        textarea.cut();
                        return Transition::Mode(Mode::Insert);
                    }
                    input => return Transition::Pending(input),
                }

                // Handle the pending operator
                match self.mode {
                    Mode::Operator('y') => {
                        textarea.copy();
                        constrain_cursor_for_normal_mode(textarea);
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('d') => {
                        textarea.cut();
                        constrain_cursor_for_normal_mode(textarea);
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('c') => {
                        textarea.cut();
                        Transition::Mode(Mode::Insert)
                    }
                    _ => Transition::Nop,
                }
            }
            Mode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => Transition::Mode(Mode::Normal),
                input => {
                    textarea.input(input); // Use default key mappings in insert mode
                    Transition::Mode(Mode::Insert)
                }
            },
        }
    }
}

/// Constrain cursor position for vim normal mode
/// In vim normal mode, cursor should be ON a character, not beyond the last character
/// Also prevents cursor from going to non-existent lines
fn constrain_cursor_for_normal_mode(textarea: &mut TextArea) {
    let (row, col) = textarea.cursor();
    let lines = textarea.lines();
    
    if lines.is_empty() {
        return;
    }
    
    // Ensure row is within bounds - prevent extra line at end
    let max_row = if lines.len() == 1 && lines[0].is_empty() {
        0 // Special case: single empty line
    } else {
        lines.len() - 1
    };
    let row = row.min(max_row);
    
    // In normal mode, cursor cannot be beyond the last character
    // For empty lines, cursor should be at position 0
    let line = &lines[row];
    let max_col = if line.is_empty() { 0 } else { line.chars().count() - 1 };
    let col = col.min(max_col);
    
    if (row, col) != textarea.cursor() {
        textarea.move_cursor(CursorMove::Jump(row as u16, col as u16));
    }
}

/// Paste below the current line (vim 'p' behavior for line-wise yanks)
/// For character-wise yanks, paste after the cursor
fn vim_paste_below(textarea: &mut TextArea) {
    match &textarea.yank_text() {
        text if text.contains('\n') => {
            // Line-wise paste: move to end of current line and paste
            // The yanked text already contains newlines, so no need to insert one
            textarea.move_cursor(CursorMove::End);
            textarea.paste();
        }
        _ => {
            // Character-wise paste: paste after cursor (move forward then paste)
            textarea.move_cursor(CursorMove::Forward);
            textarea.paste();
        }
    }
}

/// Paste above the current line (vim 'P' behavior for line-wise yanks)
/// For character-wise yanks, paste before the cursor
fn vim_paste_above(textarea: &mut TextArea) {
    match &textarea.yank_text() {
        text if text.contains('\n') => {
            textarea.move_cursor(CursorMove::Head);
            textarea.paste();
        }
        _ => {
            textarea.paste();
        }
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = if let Some(path) = env::args().nth(1) {
        let file = fs::File::open(path)?;
        io::BufReader::new(file)
            .lines()
            .collect::<io::Result<_>>()?
    } else {
        TextArea::default()
    };

    textarea.set_block(Mode::Normal.block());
    textarea.set_cursor_style(Mode::Normal.cursor_style());
    let mut vim = Vim::new(Mode::Normal);
    
    // Apply initial cursor constraints for normal mode
    constrain_cursor_for_normal_mode(&mut textarea);

    loop {
        term.draw(|f| f.render_widget(&textarea, f.area()))?;

        vim = match vim.transition(crossterm::event::read()?.into(), &mut textarea) {
            Transition::Mode(mode) if vim.mode != mode => {
                textarea.set_block(mode.block());
                textarea.set_cursor_style(mode.cursor_style());
                if mode == Mode::Normal {
                    constrain_cursor_for_normal_mode(&mut textarea);
                }
                Vim::new(mode)
            }
            Transition::Nop | Transition::Mode(_) => vim,
            Transition::Pending(input) => vim.with_pending(input),
            Transition::Quit => break,
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Lines: {:?}", textarea.lines());

    Ok(())
}
