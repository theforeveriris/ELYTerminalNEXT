//! The new-tab button: a plus that asks which program to open.

use ely_gpui_component::{
    buttons::{ButtonVariant, IconButton},
    menus::{ContextMenu, Menu, MenuItem},
    primitives::IconName,
    theme::ControlSize,
};
use gpui::{App, Entity, IntoElement, ParentElement};

use super::Workspace;
use crate::settings::language;

/// The plus: a press asks for the menu at that point, which lists the session the settings start, every shell the scan found, and every ssh host.
pub(super) fn button(
    workspace: &Entity<Workspace>,
    request: Option<(u64, gpui::Point<gpui::Pixels>)>,
    cx: &App,
) -> impl IntoElement + use<> {
    let adder = workspace.clone();
    ContextMenu::new("new-tab", menu(workspace, cx))
        .manual(request)
        .child(
            IconButton::new("new-tab-plus", IconName::Plus)
                .variant(ButtonVariant::Secondary)
                .size(ControlSize::Sm)
                .tooltip(language::message(cx, "workspace.new_tab"))
                .on_click(move |event, _, cx| {
                    let at = event.position();
                    adder.update(cx, |this, cx| this.ask_add_menu(at, cx))
                }),
        )
}

/// What the plus lists, in the order a hand reaches for it: the default session, then the shells, then the hosts.
fn menu(workspace: &Entity<Workspace>, cx: &App) -> Menu {
    let mut menu = Menu::new();
    let adder = workspace.clone();
    menu = menu.item(
        MenuItem::new(language::message(cx, "workspace.new_tab"))
            .on_click(move |window, cx| adder.update(cx, |this, cx| this.new_tab(window, cx))),
    );
    let shells = cx.global::<crate::term::shell::Shells>().0.clone();
    if !shells.is_empty() {
        menu = menu.separator();
    }
    for found in shells {
        let label = match &found.version {
            Some(version) => format!("{} ({version})", found.name),
            None => found.name.clone(),
        };
        let adder = workspace.clone();
        menu = menu.item(MenuItem::new(label).on_click(move |window, cx| {
            adder.update(cx, |this, cx| this.new_tab_with(&found, window, cx))
        }));
    }
    let hosts = cx.global::<crate::ssh::Hosts>().hosts.clone();
    if !hosts.is_empty() {
        menu = menu.separator();
    }
    for host in hosts {
        let name = host.name.clone();
        let adder = workspace.clone();
        menu = menu.item(
            MenuItem::new(host.name.clone()).on_click(move |window, cx| {
                adder.update(cx, |this, cx| this.connect(name.clone(), window, cx))
            }),
        );
    }
    menu
}
