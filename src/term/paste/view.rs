//! Looking at a paste before it goes to the program.

use ely_gpui_component::{
    buttons::Button,
    theme::{ActiveTheme, ControlSize},
};
use gpui::{Context, IntoElement, ParentElement, Styled, div};

use super::card::{LONG, lines, preview, typed, worth_a_look};
use crate::term::view::TerminalView;

impl TerminalView {
    /// Pastes `text` at once, or after a look when it holds an escape or runs long.
    pub(crate) fn offer_paste(&mut self, text: &str, cx: &mut Context<Self>) {
        if !worth_a_look(text, LONG) {
            self.paste_text(text, cx);
            return;
        }
        log::info!(
            "terminal {}: a paste of {} lines waits to be seen",
            self.pty.pid,
            lines(text)
        );
        self.pending = Some(typed(text));
        cx.notify();
    }

    /// Sends what waited, or drops it.
    pub(crate) fn settle_paste(&mut self, send: bool, cx: &mut Context<Self>) {
        let Some(text) = self.pending.take() else {
            return;
        };
        if send {
            self.paste_text(&text, cx);
        } else {
            log::info!("terminal {}: the paste is dropped", self.pty.pid);
        }
        cx.notify();
    }

    /// The card a waiting paste shows: its first lines, how many lines, and Paste or Cancel.
    pub(crate) fn paste_card(&self, cx: &Context<Self>) -> Option<impl IntoElement + use<>> {
        let text = self.pending.as_ref()?;
        let shown = preview(text, 600);
        let heading = format!("Paste {} lines?", lines(text));
        let this = cx.entity();
        let paste = this.clone();
        let cancel = this.clone();
        let actions = div()
            .flex()
            .gap_1()
            .child(
                Button::new("paste-anyway", "Paste")
                    .primary()
                    .size(ControlSize::Sm)
                    .on_click(move |_, _, cx| {
                        paste.update(cx, |view, cx| view.settle_paste(true, cx))
                    }),
            )
            .child(
                Button::new("paste-cancel", "Cancel")
                    .size(ControlSize::Sm)
                    .on_click(move |_, _, cx| {
                        cancel.update(cx, |view, cx| view.settle_paste(false, cx))
                    }),
            );
        let theme = cx.theme();
        Some(
            div()
                .absolute()
                .top_2()
                .left_2()
                .w_96()
                .p_2()
                .rounded_md()
                .bg(theme.colors.surface)
                .border_1()
                .border_color(theme.colors.border)
                .flex()
                .flex_col()
                .gap_2()
                .child(heading)
                .child(div().max_h_48().overflow_hidden().text_xs().child(shown))
                .child(actions),
        )
    }
}
