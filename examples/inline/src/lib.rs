use futures::stream::StreamExt;
use rand::{distributions::Uniform, prelude::Distribution};
use ratatui::{prelude::*, widgets::*};
use ratatui_wasm::EventStream;
#[cfg(target_arch = "wasm32")]
use ratatui_wasm::{init_terminal, CrosstermBackend, TerminalHandle};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    io,
};
#[cfg(not(target_arch = "wasm32"))]
use tokio::spawn;
use tokio::sync::mpsc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local as spawn;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use xterm_js_rs::Theme;

#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NUM_DOWNLOADS: usize = 10;

type DownloadId = usize;
type WorkerId = usize;

enum Event {
    Input(crossterm::event::KeyEvent),
    Tick,
    Resize,
    DownloadUpdate(WorkerId, DownloadId, f64),
    DownloadDone(WorkerId, DownloadId),
}

struct Downloads {
    pending: VecDeque<Download>,
    in_progress: BTreeMap<WorkerId, DownloadInProgress>,
}

impl Downloads {
    fn next(&mut self, worker_id: WorkerId) -> Option<Download> {
        match self.pending.pop_front() {
            Some(d) => {
                self.in_progress.insert(
                    worker_id,
                    DownloadInProgress {
                        id: d.id,
                        started_at: now(),
                        progress: 0.0,
                    },
                );
                Some(d)
            }
            None => None,
        }
    }
}

struct DownloadInProgress {
    id: DownloadId,
    started_at: f64,
    progress: f64,
}

struct Download {
    id: DownloadId,
    size: usize,
}

struct Worker {
    id: WorkerId,
    tx: mpsc::Sender<Download>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("terminal")
        .unwrap();

    init_terminal(
        xterm_js_rs::TerminalOptions::new()
            .with_rows(50)
            .with_cursor_blink(true)
            .with_cursor_width(10)
            .with_font_size(20)
            .with_draw_bold_text_in_bright_colors(true)
            .with_right_click_selects_word(true)
            .with_theme(
                Theme::new()
                    .with_foreground("#98FB98")
                    .with_background("#000000"),
            ),
        elem.dyn_into()?,
    );
    let handle = TerminalHandle::default();

    run(handle, CrosstermBackend::new).await.unwrap();
    Ok(())
}

pub async fn run<W, F, B>(out: W, create_backend: F) -> Result<(), Box<dyn Error>>
where
    W: io::Write,
    B: Backend,
    F: FnOnce(W) -> B,
{
    crossterm::terminal::enable_raw_mode()?;

    let backend = create_backend(out);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )
    .unwrap();

    let (tx, rx) = mpsc::channel(32);
    input_handling(tx.clone());
    let mut workers = workers(tx);
    let mut downloads = downloads();

    for w in &mut workers {
        let d = downloads.next(w.id).unwrap();
        w.tx.send(d).await.ok();
    }

    run_app(&mut terminal, workers, downloads, rx)
        .await
        .unwrap();

    crossterm::terminal::disable_raw_mode()?;
    terminal.clear().unwrap();

    Ok(())
}

fn input_handling(tx: mpsc::Sender<Event>) {
    spawn(async move {
        let mut events = EventStream::default();
        loop {
            tokio::select! {
                _ = sleep(200) => {
                    tx.send(Event::Tick).await.ok();
                }
                event = events.next() => {
                    if let Some(Ok(event)) = event {
                        match event {
                            crossterm::event::Event::Key(key) => {
                                tx.send(Event::Input(key)).await.ok();
                            }
                            crossterm::event::Event::Resize(_, _) => {
                                tx.send(Event::Resize).await.ok();
                            }
                            _ => {}
                        }
                    }

                }
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn sleep(ms: i32) {
    let fut: JsFuture = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    })
    .into();
    fut.await.unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep(ms: i32) {
    tokio::time::sleep(Duration::from_millis(ms as u64)).await
}

#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64
}

fn workers(tx: mpsc::Sender<Event>) -> Vec<Worker> {
    (0..4)
        .map(|id| {
            let (worker_tx, mut worker_rx) = mpsc::channel::<Download>(32);
            let tx = tx.clone();
            spawn(async move {
                while let Some(download) = worker_rx.recv().await {
                    let mut remaining = download.size;
                    while remaining > 0 {
                        let wait = (remaining as u64).min(10);
                        sleep((wait * 10) as i32).await;

                        remaining = remaining.saturating_sub(10);
                        let progress = (download.size - remaining) * 100 / download.size;
                        tx.send(Event::DownloadUpdate(id, download.id, progress as f64))
                            .await
                            .ok();
                    }
                    tx.send(Event::DownloadDone(id, download.id)).await.ok();
                }
            });
            Worker { id, tx: worker_tx }
        })
        .collect()
}

fn downloads() -> Downloads {
    let distribution = Uniform::new(0, 1000);
    let mut rng = rand::thread_rng();
    let pending = (0..NUM_DOWNLOADS)
        .map(|id| {
            let size = distribution.sample(&mut rng);
            Download { id, size }
        })
        .collect();
    Downloads {
        pending,
        in_progress: BTreeMap::new(),
    }
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    workers: Vec<Worker>,
    mut downloads: Downloads,
    mut rx: mpsc::Receiver<Event>,
) -> Result<(), Box<dyn Error>> {
    let mut redraw = true;
    loop {
        if redraw {
            terminal.draw(|f| ui(f, &downloads))?;
        }
        redraw = true;

        match rx.recv().await.unwrap() {
            Event::Input(event) => {
                if event.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
            Event::Resize => {
                terminal.autoresize()?;
            }
            Event::Tick => {}
            Event::DownloadUpdate(worker_id, _download_id, progress) => {
                let download = downloads.in_progress.get_mut(&worker_id).unwrap();
                download.progress = progress;
                redraw = false
            }
            Event::DownloadDone(worker_id, download_id) => {
                let download = downloads.in_progress.remove(&worker_id).unwrap();
                terminal.insert_before(1, |buf| {
                    Paragraph::new(Line::from(vec![
                        Span::from("Finished "),
                        Span::styled(
                            format!("download {download_id}"),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::from(format!(" in {}ms", now() - download.started_at)),
                    ]))
                    .render(buf.area, buf);
                })?;
                match downloads.next(worker_id) {
                    Some(d) => {
                        workers[worker_id].tx.send(d).await.ok();
                    }
                    None => {
                        if downloads.in_progress.is_empty() {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new("Done !").render(buf.area, buf);
                            })?;
                            break;
                        }
                    }
                };
            }
        };
    }
    Ok(())
}

fn ui(f: &mut Frame, downloads: &Downloads) {
    let size = f.size();

    let block = Block::default().title(block::Title::from("Progress").alignment(Alignment::Center));
    f.render_widget(block, size);

    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(2), Constraint::Length(4)])
        .margin(1)
        .split(size);

    // total progress
    let done = NUM_DOWNLOADS - downloads.pending.len() - downloads.in_progress.len();
    let progress = LineGauge::default()
        .gauge_style(Style::default().fg(Color::Blue))
        .label(format!("{done}/{NUM_DOWNLOADS}"))
        .ratio(done as f64 / NUM_DOWNLOADS as f64);
    f.render_widget(progress, chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[1]);

    // in progress downloads
    let items: Vec<ListItem> = downloads
        .in_progress
        .values()
        .map(|download| {
            ListItem::new(Line::from(vec![
                Span::raw(symbols::DOT),
                Span::styled(
                    format!(" download {:>2}", download.id),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" ({}ms)", now() - download.started_at)),
            ]))
        })
        .collect();
    let list = List::new(items);
    f.render_widget(list, chunks[0]);

    for (i, (_, download)) in downloads.in_progress.iter().enumerate() {
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(download.progress / 100.0);
        if chunks[1].top().saturating_add(i as u16) > size.bottom() {
            continue;
        }
        f.render_widget(
            gauge,
            Rect {
                x: chunks[1].left(),
                y: chunks[1].top().saturating_add(i as u16),
                width: chunks[1].width,
                height: 1,
            },
        );
    }
}
