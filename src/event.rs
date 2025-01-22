use std::io;
use std::task::{Poll, ready};

use futures::Stream;
use terminput::Event;

use crate::poll_next_event;

#[derive(Default)]
pub struct EventStream {}

impl EventStream {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stream for EventStream {
    type Item = io::Result<crossterm::event::Event>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            if let Some(event) = ready!(poll_next_event(cx)) {
                match Event::parse_from(event.as_bytes()) {
                    Ok(Some(e)) => {
                        if let Ok(e) = e.try_into() {
                            return Poll::Ready(Some(Ok(e)));
                        }
                    }
                    Err(e) => {
                        return Poll::Ready(Some(Err(e)));
                    }
                    _ => {
                        continue;
                    }
                }
            } else {
                return Poll::Pending;
            }
        }
    }
}
