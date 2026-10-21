//! The small window a theme grid shows, in that theme's colors.

use ely_gpui_component::theme::{ActiveTheme, Mix, Radius};
use gpui::{App, Div, Hsla, ParentElement, Styled, div, px, relative};

use crate::themes::TerminalTheme;

/// A small window in `theme`'s colors: the sessions strip, then a prompt, its output and the cursor.
pub(super) fn preview(theme: &TerminalTheme, cx: &App) -> Div {
    let (bg, fg) = (theme.background, theme.foreground);
    let bar = |color: Hsla, width: f32| div().h_1().w(relative(width)).rounded_full().bg(color);
    let prompt = |end: Div| {
        let dot = div()
            .flex_none()
            .size_1p5()
            .rounded_full()
            .bg(theme.ansi[2]);
        div().flex().items_center().gap_1().child(dot).child(end)
    };
    let strip = div()
        .w(relative(0.2))
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .pt_2()
        .bg(bg.mix(&fg, 0.06))
        .children([0.6, 0.3, 0.3].map(|alpha| bar(fg.opacity(alpha), 0.6)));
    let lines = div()
        .flex_1()
        .flex()
        .flex_col()
        .justify_center()
        .gap_1()
        .px_2()
        .child(prompt(bar(fg, 0.5)))
        .child(bar(theme.ansi[4], 0.75))
        .child(bar(theme.ansi[5], 0.55))
        .child(prompt(div().flex_none().w_1p5().h_2p5().bg(theme.cursor)));
    let chrome = cx.theme();
    div()
        .w_full()
        .h(px(72.))
        .flex()
        .overflow_hidden()
        .rounded(chrome.radius(Radius::Sm))
        .border_1()
        .border_color(chrome.colors.border)
        .bg(bg)
        .child(strip)
        .child(lines)
}
