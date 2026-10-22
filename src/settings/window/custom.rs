//! The theme of the user's own: a picker for each color role, grouped by the part it colors.

use ely_gpui_component::{
    buttons::Button,
    forms::{ColorPicker, Input},
    layout::SimpleGrid,
    settings::SettingsSection,
    theme::ControlSize,
};
use gpui::{App, Div, Entity, ParentElement, Styled, Window, div, px, rems};

use super::SettingsWindow;
use crate::{
    settings::{SettingsStore, language},
    themes::{self, custom},
};

impl SettingsWindow {
    /// The theme of the user's own: a picker for each color role, four groups of them.
    pub(super) fn custom(&self, this: &Entity<Self>, window: &Window, cx: &App) -> Div {
        let colors = SettingsStore::get(cx).custom_colors.clone();
        let rem = window.rem_size();
        // A cell at least this wide, so the grid fits as many as the window holds.
        let (min, gap) = (px(120.), rems(0.75).to_pixels(rem));
        let mut out = div().flex().flex_col().gap_6().child(
            SettingsSection::new(language::message(cx, "settings.window.custom"))
                .row(start_from_theme(this, cx)),
        );
        for (group, roles) in custom::GROUPS {
            let mut grid = SimpleGrid::new(format!("custom-{group}"), min, gap);
            for role in roles {
                let Some(&color) = colors.get(role).as_ref() else {
                    continue;
                };
                let picker = self.picker(this, role, color);
                grid = grid.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().child(role.replace('_', " ")))
                        .child(picker)
                        .child(div().text_xs().child(custom::hex_of(color))),
                );
            }
            out = out.child(
                SettingsSection::new(language::message(cx, &format!("settings.custom.{group}")))
                    .row(grid),
            );
        }
        out.child(self.saved_themes(this, cx))
    }

    /// The themes the user saved: a name to save the colors under, then each one kept, with a way to forget it.
    fn saved_themes(&self, this: &Entity<Self>, cx: &App) -> SettingsSection {
        let settings = SettingsStore::get(cx);
        let save = {
            let this = this.clone();
            Button::new("custom-save", language::message(cx, "settings.custom.save"))
                .size(ControlSize::Sm)
                .on_click(move |_, _, cx| save_theme(&this, cx))
        };
        let mut section = SettingsSection::new(language::message(cx, "settings.custom.saved")).row(
            div()
                .flex()
                .items_center()
                .gap_2()
                .w(px(320.))
                .child(
                    div()
                        .flex_1()
                        .child(Input::new(&self.theme_name).size(ControlSize::Sm)),
                )
                .child(save),
        );
        for theme in &settings.saved_themes {
            let id = theme.id.clone();
            let this = this.clone();
            let forget = Button::new(
                format!("forget-{}", theme.id),
                language::message(cx, "settings.custom.delete"),
            )
            .size(ControlSize::Sm)
            .on_click(move |_, _, cx| forget_theme(&this, &id, cx));
            section = section.row(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_sm().child(theme.name.clone()))
                    .child(forget),
            );
        }
        section
    }

    /// One role's picker, which writes its color into the settings as it changes.
    fn picker(&self, this: &Entity<Self>, role: &'static str, color: gpui::Hsla) -> ColorPicker {
        let this = this.clone();
        ColorPicker::new(format!("custom-{role}"), color).on_change(move |picked, _, cx| {
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
        })
    }
}

/// A button that fills the user's colors with the theme in use, so an edit starts from what is on screen.
fn start_from_theme(this: &Entity<SettingsWindow>, cx: &App) -> Button {
    let this = this.clone();
    Button::new(
        "custom-from-theme",
        language::message(cx, "settings.custom.from_theme"),
    )
    .size(ControlSize::Sm)
    .on_click(move |_, window, cx| {
        let settings = SettingsStore::get(cx);
        let worn = themes::active(settings, window.appearance(), cx);
        let next = custom::Colors::from_theme(&worn);
        if super::set(
            &this,
            format!("the colors of {}", worn.name),
            cx,
            |settings| settings.custom_colors = next,
        ) {
            log::info!("settings: the custom theme starts from {}", worn.id);
        }
    })
}

/// Saves the colors now in the editor under the name in the field, and wears the result.
fn save_theme(this: &Entity<SettingsWindow>, cx: &mut App) {
    let name = this.read(cx).theme_name.read(cx).text().trim().to_string();
    if name.is_empty() {
        return;
    }
    let settings = SettingsStore::get(cx);
    let theme = custom::SavedTheme::new(
        &name,
        settings.custom_colors.clone(),
        &settings.saved_themes,
    );
    // Saving keeps the editor on the colors it has: the new theme is a card to wear, not a switch.
    if super::set(this, format!("saved theme {name}"), cx, |settings| {
        settings.saved_themes.push(theme);
    }) {
        this.update(cx, |window, cx| {
            window
                .theme_name
                .update(cx, |field, cx| field.set_text("", cx));
        });
    }
    log::info!("settings: saved theme {name}");
}

/// Forgets the theme `id`, and puts the custom theme on if it was the one worn.
fn forget_theme(this: &Entity<SettingsWindow>, id: &str, cx: &mut App) {
    let change = format!("forgot theme {id}");
    if super::set(this, change, cx, |settings| {
        settings.saved_themes.retain(|theme| theme.id != id);
        if settings.theme == id {
            settings.theme = themes::CUSTOM.to_string();
        }
        if settings.dark_theme.as_deref() == Some(id) {
            settings.dark_theme = None;
        }
    }) {
        log::info!("settings: forgot theme {id}");
    }
}
