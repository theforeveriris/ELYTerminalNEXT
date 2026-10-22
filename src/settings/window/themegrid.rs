//! The theme grid: a card for each theme.

use ely_gpui_component::{
    forms::RadioCard,
    theme::{ActiveTheme, TextSize},
};
use gpui::{App, Div, Entity, SharedString, Styled, Window, div, prelude::*, rems};

use super::{SettingsWindow, set};
use crate::{settings::Settings, themes};

/// Omarchy's theme where followed, then every built-in, as cards; `dark` picks the dark system's.
pub(super) fn grid(
    this: &Entity<SettingsWindow>,
    title: Option<String>,
    dark: bool,
    settings: &Settings,
    window: &Window,
    cx: &App,
) -> Div {
    let theme = cx.theme();
    let (field, chosen) = if dark {
        ("dark_theme", settings.dark_theme.as_deref())
    } else {
        ("theme", Some(settings.theme.as_str()))
    };
    let omarchy = cx.try_global::<themes::Omarchy>().map(|omarchy| &omarchy.0);
    // The theme of the user's own colors, alongside the built-in ones, then the ones they saved.
    let custom = themes::custom(&settings.custom_colors);
    let saved: Vec<themes::TerminalTheme> =
        settings.saved_themes.iter().map(themes::saved).collect();
    let cards = omarchy
        .into_iter()
        .chain(themes::all())
        .chain(saved.iter())
        .map(|each| {
            let id = each.id.clone();
            let chosen_here = chosen == Some(id.as_ref());
            let this = this.clone();
            let key = SharedString::from(format!("{field}-{id}"));
            RadioCard::new(key, chosen_here, each.name.clone())
                .preview(super::preview::preview(each, cx))
                .on_select(move |_, cx| {
                    set(&this, format!("{field} is {id}"), cx, |settings| {
                        if dark {
                            settings.dark_theme = Some(id.to_string());
                        } else {
                            settings.theme = id.to_string();
                        }
                    });
                })
        })
        .chain(std::iter::once({
            let id = themes::CUSTOM;
            let this = this.clone();
            let key = SharedString::from(format!("{field}-{id}"));
            RadioCard::new(key, chosen == Some(id), custom.name.clone())
                .preview(super::preview::preview(&custom, cx))
                .on_select(move |_, cx| {
                    set(&this, format!("{field} is {id}"), cx, |settings| {
                        if dark {
                            settings.dark_theme = Some(id.into());
                        } else {
                            settings.theme = id.into();
                        }
                    });
                })
        }));
    let rem = window.rem_size();
    let gap = rems(0.75).to_pixels(rem);
    // A fixed number of columns, not a measured one: a grid that measures its own width to pick its columns never settles, and the window redraws itself forever.
    div()
        .flex()
        .flex_col()
        .gap_2()
        .py_3()
        .border_b_1()
        .border_color(theme.colors.border)
        .children(title.map(|title| div().text_size(theme.text_size(TextSize::Sm)).child(title)))
        .child(div().grid().grid_cols(COLUMNS).gap(gap).children(cards))
}

/// The theme cards' columns; the window is 900 wide, so three fit.
const COLUMNS: u16 = 3;
