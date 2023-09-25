use std::{
    io,
    task::{ready, Poll},
};

use crossterm::event::Event;
use futures::Stream;

use crate::{parse::parse_event, poll_next_event};

#[derive(Default)]
pub struct EventStream {}

impl EventStream {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stream for EventStream {
    type Item = io::Result<Event>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            if let Some(event) = ready!(poll_next_event(cx)) {
                match parse_event(event.as_bytes(), false) {
                    Ok(Some(e)) => {
                        return Poll::Ready(Some(Ok(e)));
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
