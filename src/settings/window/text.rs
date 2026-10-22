//! A text field that writes what it holds into its setting, and draws the page again once it has.

use ely_gpui_component::forms::{InputEvent, TextInput};
use gpui::{AppContext, Context, Entity, Subscription, Window};

use super::SettingsWindow;
use crate::settings::{Settings, SettingsStore};

/// A field over `text`, whose changes `save` writes; the field and the subscription that keeps it.
pub(super) fn field(
    window: &mut Window,
    cx: &mut Context<SettingsWindow>,
    id: &'static str,
    placeholder: String,
    text: String,
    save: fn(&mut Settings, String),
) -> (Entity<TextInput>, Subscription) {
    let input = cx.new(|cx| {
        let mut input = TextInput::new(window, cx).placeholder(placeholder);
        input.set_text(text, cx);
        input
    });
    let typed = cx.subscribe(&input, move |this, input, event: &InputEvent, cx| {
        if let InputEvent::Changed = event {
            let value = input.read(cx).text().to_string();
            if let Err(error) = SettingsStore::update(cx, |settings| save(settings, value.clone()))
            {
                log::debug!("settings: {id} stays: {error:#}");
            }
            this.paged(cx);
        }
    });
    (input, typed)
}
