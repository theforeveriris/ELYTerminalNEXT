//! The theme of the user's own: a picker for each color role.

use ely_gpui_component::{forms::ColorPicker, settings::SettingsSection};
use gpui::{App, Div, Entity, ParentElement, Styled, Window, div, rems};

use super::SettingsWindow;
use crate::{settings::SettingsStore, themes::custom::ROLES};

impl SettingsWindow {
    /// The theme of the user's own: a picker for each color role.
    pub(super) fn custom(&self, this: &Entity<Self>, window: &Window, cx: &App) -> Div {
        let colors = SettingsStore::get(cx).custom_colors.clone();
        let gap = rems(0.75).to_pixels(window.rem_size());
        let mut grid = div().grid().grid_cols(ROLES.len() as u16).gap(gap);
        for (at, role) in ROLES.iter().enumerate() {
            let Some(color) = colors.get(role) else {
                continue;
            };
            let this = this.clone();
            let role = *role;
            let picker = ColorPicker::new(("custom", at), color).on_change(move |picked, _, cx| {
                let mut next = SettingsStore::get(cx).custom_colors.clone();
                if !next.set(role, picked) {
                    return;
                }
                super::set(
                    &this,
                    format!("the color of {role} is {picked:?}"),
                    cx,
                    |settings| settings.custom_colors = next,
                );
            });
            grid = grid.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_sm().child(role.replace('_', " ")))
                    .child(picker),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(SettingsSection::new("Custom theme").row(grid))
    }
}
