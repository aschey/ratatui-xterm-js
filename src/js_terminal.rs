use std::cell::{OnceCell, RefCell};
use std::io;
use std::sync::{Mutex, OnceLock};
use std::task::{Context, Poll};

use crossterm::terminal::WindowSize;
use futures::StreamExt;
use futures::channel::mpsc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlElement;
use xterm_js_rs::addons::fit::FitAddon;

thread_local! {
    static TERMINAL: OnceCell<xterm_js_rs::Terminal> = const { OnceCell::new() };
}

static DATA_CHANNEL: OnceLock<Mutex<mpsc::Receiver<String>>> = OnceLock::new();

pub(crate) fn with_terminal<F, T>(f: F) -> T
where
    F: FnOnce(&xterm_js_rs::Terminal) -> T,
{
    TERMINAL.with(|t| f(t.get().unwrap()))
}

pub fn init_terminal(options: &xterm_js_rs::TerminalOptions, parent: HtmlElement) {
    TERMINAL.with(|t| {
        let (mut tx, rx) = mpsc::channel(32);
        let mut tx_ = tx.clone();
        let terminal = xterm_js_rs::Terminal::new(options);

        let callback = Closure::wrap(Box::new(move |e: xterm_js_rs::Event| {
            tx_.try_send(e.as_string().unwrap()).ok();
        }) as Box<dyn FnMut(_)>);
        terminal.on_data(callback.as_ref().unchecked_ref());
        callback.forget();

        let callback = Closure::wrap(Box::new(move |e: xterm_js_rs::Event| {
            tx.try_send(e.as_string().unwrap()).ok();
        }) as Box<dyn FnMut(_)>);
        terminal.on_binary(callback.as_ref().unchecked_ref());
        callback.forget();

        DATA_CHANNEL.set(Mutex::new(rx)).unwrap();

        let addon = FitAddon::new();
        terminal.load_addon(addon.clone().dyn_into::<FitAddon>().unwrap().into());
        addon.fit();

        terminal.open(parent);
        terminal.focus();
        if t.set(terminal).is_err() {
            panic!();
        }
    });
}

pub(crate) fn poll_next_event(cx: &mut Context<'_>) -> Poll<Option<String>> {
    DATA_CHANNEL
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .poll_next_unpin(cx)
}

pub fn window_size() -> io::Result<WindowSize> {
    Ok(with_terminal(|t| WindowSize {
        rows: t.get_rows() as u16,
        columns: t.get_cols() as u16,
        width: t.get_element().client_width() as u16,
        height: t.get_element().client_height() as u16,
    }))
}

pub(crate) fn size() -> io::Result<(u16, u16)> {
    window_size().map(|s| (s.columns, s.rows))
}

pub fn cursor_position() -> io::Result<(u16, u16)> {
    Ok(with_terminal(|t| {
        let active = t.get_buffer().get_active();
        (active.get_cursor_x() as u16, active.get_cursor_y() as u16)
    }))
}

#[derive(Default)]
pub struct TerminalHandle {
    buffer: RefCell<Vec<u8>>,
}

impl io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let s = String::from_utf8(self.buffer.replace(Vec::new()))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        with_terminal(|t| t.write(&s));
        Ok(())
    }
}
