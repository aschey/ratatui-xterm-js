use std::error::Error;
use std::io;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    simple::run(stdout, ratatui::backend::CrosstermBackend::new).await
}
