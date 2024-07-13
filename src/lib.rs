#[cfg(target_arch = "wasm32")]
pub use backend::XtermJsBackend;
#[cfg(target_arch = "wasm32")]
pub use event::EventStream;
#[cfg(target_arch = "wasm32")]
pub use js_terminal::*;
#[cfg(target_arch = "wasm32")]
pub use xterm_js_rs as xterm;

#[cfg(target_arch = "wasm32")]
mod backend;
#[cfg(target_arch = "wasm32")]
mod event;
#[cfg(target_arch = "wasm32")]
mod js_terminal;
