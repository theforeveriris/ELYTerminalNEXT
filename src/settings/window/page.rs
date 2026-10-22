//! The settings page: its rows, drawn apart from the window's frame so a scroll reuses them, scrolled inside its own box rather than the window's.

use gpui::{
    AnyElement, App, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::{InlineMessage, SettingsWindow, Severity};

pub struct PageView {
    pub(super) settings: Entity<SettingsWindow>,
}

impl Render for PageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.settings.clone();
        let (rows, scroll) = {
            let settings = settings.read(cx);
            (
                settings.page(&self.settings, window, cx),
                settings.page_scroll.clone(),
            )
        };
        // The page is the scroll: its own box is the window's height, and its rows run past it.
        div()
            .id("settings-page")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .child(rows)
    }
}

impl SettingsWindow {
    /// The open page, with the failure it shows above it. Drawn by `PageView`, so a scroll of the window reuses it.
    fn page(&self, this: &Entity<SettingsWindow>, window: &Window, cx: &App) -> AnyElement {
        let start = std::time::Instant::now();
        let page = match self.section.as_ref() {
            "appearance" => self.appearance(this, window, cx).into_any_element(),
            "terminal" => self.terminal(this, cx).into_any_element(),
            "general" => self.general(this, cx).into_any_element(),
            other => unreachable!("settings: no section {other}"),
        };
        let built = div()
            .flex()
            .flex_col()
            .gap_4()
            // Its rows are the content a scroll moves, so it must not shrink to the box it sits in.
            .flex_shrink_0()
            .children((self.failed.clone()).map(|why| InlineMessage::new(Severity::Danger, why)))
            .child(page)
            .into_any_element();
        log::debug!("settings: {} built in {:?}", self.section, start.elapsed());
        built
    }
}
