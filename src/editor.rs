use std::env;
use std::result::Result;

use termion::event::Key;

use crate::Document;
use crate::Row;
use crate::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
}

impl Editor {
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(file_name).unwrap_or_default()
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal."),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }

            if self.should_quit {
                break;
            }

            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(&self.cursor_position);
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(height) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let size = self.terminal.size();
        let height = size.height.saturating_sub(1) as usize;
        let width = size.width.saturating_sub(1) as usize;
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => x = x.saturating_sub(1),
            Key::Right => {
                if x < width {
                    x = x.saturating_add(1);
                }
            }
            Key::PageUp => y = 0,
            Key::PageDown => y = height,
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        self.cursor_position = Position { x, y }
    }

    fn draw_welcome_msg(&self) {
        let mut welcome_msg = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_msg.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{}{}", spaces, welcome_msg);
        welcome_msg.truncate(width);
        println!("{}\r", welcome_msg);
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height - 1 {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_msg();
            } else {
                println!("~\r");
            }
        }
    }
}

fn die(e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
