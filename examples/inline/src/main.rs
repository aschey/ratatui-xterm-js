use std::{error::Error, io};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    inline::run(stdout, ratatui::backend::CrosstermBackend::new).await
}
