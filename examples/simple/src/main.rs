use std::{error::Error, io};

use ratatui_wasm::CrosstermBackend;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    simple::run(stdout, CrosstermBackend::new).await
}
