//! This module provides the `CrosstermBackend` implementation for the `Backend` trait.
//! It uses the `crossterm` crate to interact with the terminal.
//!
//!
//! [`Backend`]: trait.Backend.html
//! [`CrosstermBackend`]: struct.CrosstermBackend.html

use std::io::{self, Write};

use ratatui::{
    backend::{Backend, ClearType, WindowSize},
    buffer::Cell,
    layout::Size,
    prelude::Rect,
};

use crate::js_terminal::{cursor_position, window_size, TerminalHandle};

/// A backend implementation using the `crossterm` crate.
///
/// The `CrosstermBackend` struct is a wrapper around a type implementing `Write`, which
/// is used to send commands to the terminal. It provides methods for drawing content,
/// manipulating the cursor, and clearing the terminal screen.
///
/// # Example
///
/// ```rust
/// use ratatui::backend::{Backend, CrosstermBackend};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buffer = std::io::stdout();
/// let mut backend = CrosstermBackend::new(buffer);
/// backend.clear()?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct CrosstermBackend {
    inner: ratatui::backend::CrosstermBackend<TerminalHandle>,
}

impl CrosstermBackend {
    /// Creates a new `CrosstermBackend` with the given buffer.
    pub fn new(handle: TerminalHandle) -> Self {
        Self {
            inner: ratatui::backend::CrosstermBackend::new(handle),
        }
    }
}

impl Write for CrosstermBackend {
    /// Writes a buffer of bytes to the underlying buffer.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    /// Flushes the underlying buffer.
    fn flush(&mut self) -> io::Result<()> {
        std::io::Write::flush(&mut self.inner)
    }
}

impl Backend for CrosstermBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.inner.draw(content)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }

    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        cursor_position()
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.inner.set_cursor(x, y)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        self.inner.clear_region(clear_type)
    }

    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        self.inner.append_lines(n)
    }

    fn size(&self) -> io::Result<Rect> {
        let (width, height) = crate::js_terminal::size()?;
        Ok(Rect::new(0, 0, width, height))
    }

    fn window_size(&mut self) -> Result<WindowSize, io::Error> {
        let crossterm::terminal::WindowSize {
            columns,
            rows,
            width,
            height,
        } = window_size()?;
        Ok(WindowSize {
            columns_rows: Size {
                width: columns,
                height: rows,
            },
            pixels: Size { width, height },
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        std::io::Write::flush(&mut self.inner)
    }
}
