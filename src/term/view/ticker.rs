//! A terminal's own clock and its fit to its box: the blink, the event pump and the resize.

use futures::StreamExt;
use gpui::{Context, Task};

use super::{BLINK, TerminalView};
use crate::term::pty::{Events, fit};

impl TerminalView {
    pub(super) fn pump(mut events: Events, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            while let Some(first) = events.next().await {
                let mut batch = vec![first];
                while let Ok(more) = events.try_recv() {
                    batch.push(more);
                }
                if view.update(cx, |view, cx| view.handle(batch, cx)).is_err() {
                    break;
                }
            }
        })
    }

    pub(super) fn blink(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor().timer(BLINK).await;
                let ticked = view.update(cx, |view, cx| {
                    if view.blinking {
                        view.shown = !view.shown;
                        cx.notify();
                    }
                });
                if ticked.is_err() {
                    break;
                }
            }
        })
    }

    /// Fits the grid to `columns` by `lines`; true when the size changed.
    pub(crate) fn fit(&mut self, columns: usize, lines: usize, cx: &mut Context<Self>) -> bool {
        let fitted = fit(columns, lines, self.cell);
        let size = (usize::from(fitted.num_cols), usize::from(fitted.num_lines));
        if size == self.size {
            return false;
        }
        self.size = size;
        self.pty.resize(size, self.cell);
        log::debug!("terminal {}: {} by {}", self.pty.pid, size.0, size.1);
        self.search(false, cx);
        true
    }
}

impl TerminalView {}
