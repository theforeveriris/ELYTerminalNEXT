//! The field that filters the sessions column.

use ely_gpui_component::forms::{InputEvent, TextInput};
use gpui::{AppContext, Context, Entity, Subscription, Window};

use super::Workspace;

/// The filter field, and what it holds in lower case.
pub(super) struct Filter {
    pub(super) field: Entity<TextInput>,
    pub(super) text: String,
    _typed: Subscription,
}

impl Filter {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Workspace>) -> Self {
        let field = cx.new(|cx| {
            TextInput::new(window, cx)
                .placeholder(crate::settings::language::message(cx, "workspace.filter"))
        });
        let typed = cx.subscribe(&field, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Changed = event {
                this.filter.text = input.read(cx).text().to_lowercase();
                cx.notify();
            }
        });
        Self {
            field,
            text: String::new(),
            _typed: typed,
        }
    }
}
