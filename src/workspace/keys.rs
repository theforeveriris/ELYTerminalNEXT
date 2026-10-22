//! The window's keys: Ghostty's, plus this fork's own, bound to the actions the workspace answers, Cmd on macOS and Ctrl-Shift or Alt elsewhere so a shell keeps the plain keys it expects.

use gpui::{App, KeyBinding};

use super::layout::Direction;
use super::{
    CONTEXT, CheckHealth, CloneSession, ClosePane, CloseSession, CloseTab, EvenPanes, FocusPane,
    LastSession, LastTab, MoveTabDown, MoveTabUp, NewSession, NewTab, NextSession, NextTab,
    NextUsed, PinTab, PreviousSession, PreviousTab, PreviousUsed, SelectSession, SelectTab,
    SplitDown, SplitRight, SwapPane, TogglePanel, ToggleSessions, Zen, ZoomPane,
};

/// Binds every key the workspace answers: Ghostty's, and the ones this fork adds beside them.
pub fn bind(cx: &mut App) {
    let (sessions, panel, new, close, digit) = if cfg!(target_os = "macos") {
        ("cmd-b", "cmd-alt-b", "cmd-t", "cmd-w", "cmd")
    } else {
        (
            "ctrl-shift-b",
            "ctrl-shift-alt-b",
            "ctrl-shift-t",
            "ctrl-shift-w",
            "alt",
        )
    };
    // The tabs and the panes, as Ghostty binds them.
    let (tab_new, tab_close, split, split_down, pane) = if cfg!(target_os = "macos") {
        ("cmd-t", "cmd-w", "cmd-d", "cmd-shift-d", "cmd-alt")
    } else {
        (
            "ctrl-shift-t",
            "ctrl-shift-w",
            "ctrl-shift-d",
            "ctrl-shift-alt-d",
            "ctrl-shift-alt",
        )
    };
    let (zoom, zen) = if cfg!(target_os = "macos") {
        ("cmd-shift-enter", "cmd-shift-z")
    } else {
        ("ctrl-shift-enter", "ctrl-shift-z")
    };
    let swap = if cfg!(target_os = "macos") {
        "cmd-alt-shift"
    } else {
        "ctrl-alt"
    };
    let tab_clone = if cfg!(target_os = "macos") {
        "cmd-shift-k"
    } else {
        "ctrl-shift-k"
    };
    let context = Some(CONTEXT);
    cx.bind_keys([
        KeyBinding::new(sessions, ToggleSessions, context),
        KeyBinding::new(panel, TogglePanel, context),
    ]);
    // The tabs take the plain keys, and the sessions the shifted ones.
    cx.bind_keys([
        KeyBinding::new(tab_new, NewTab, context),
        KeyBinding::new(tab_close, CloseTab, context),
        KeyBinding::new(split, SplitRight, context),
        KeyBinding::new(split_down, SplitDown, context),
        KeyBinding::new(&format!("{pane}-w"), ClosePane, context),
        KeyBinding::new(zoom, ZoomPane, context),
        KeyBinding::new(zen, Zen, context),
        // A clone of the session, a report on the machine, and the panes evened out again.
        KeyBinding::new(tab_clone, CloneSession, context),
        KeyBinding::new("ctrl-shift-alt-h", CheckHealth, context),
        KeyBinding::new("ctrl-shift-alt-e", EvenPanes, context),
        // TAB-03 and TAB-04: the tab moves among its neighbours, or stays where it is for good.
        KeyBinding::new("ctrl-shift-alt-up", MoveTabUp, context),
        KeyBinding::new("ctrl-shift-alt-down", MoveTabDown, context),
        KeyBinding::new("ctrl-shift-alt-p", PinTab, context),
    ]);
    cx.bind_keys([
        KeyBinding::new(new, NewSession, context),
        KeyBinding::new(close, CloseSession, context),
        KeyBinding::new("ctrl-tab", NextUsed, context),
        KeyBinding::new("ctrl-shift-tab", PreviousUsed, context),
        KeyBinding::new("ctrl-pagedown", NextTab, context),
        KeyBinding::new("ctrl-pageup", PreviousTab, context),
        KeyBinding::new(&format!("{digit}-9"), LastTab, context),
        // The sessions step within the shown tab.
        KeyBinding::new("ctrl-shift-right", NextSession, context),
        KeyBinding::new("ctrl-shift-left", PreviousSession, context),
        KeyBinding::new(&format!("{digit}-shift-9"), LastSession, context),
    ]);
    cx.bind_keys(
        (1..=8).map(|n| KeyBinding::new(&format!("{digit}-{n}"), SelectTab(n - 1), context)),
    );
    cx.bind_keys(
        (1..=8)
            .map(|n| KeyBinding::new(&format!("{digit}-shift-{n}"), SelectSession(n - 1), context)),
    );
    cx.bind_keys([
        KeyBinding::new(&format!("{pane}-left"), FocusPane(Direction::Left), context),
        KeyBinding::new(
            &format!("{pane}-right"),
            FocusPane(Direction::Right),
            context,
        ),
        KeyBinding::new(&format!("{pane}-up"), FocusPane(Direction::Up), context),
        KeyBinding::new(&format!("{pane}-down"), FocusPane(Direction::Down), context),
        KeyBinding::new(&format!("{swap}-left"), SwapPane(Direction::Left), context),
        KeyBinding::new(
            &format!("{swap}-right"),
            SwapPane(Direction::Right),
            context,
        ),
        KeyBinding::new(&format!("{swap}-up"), SwapPane(Direction::Up), context),
        KeyBinding::new(&format!("{swap}-down"), SwapPane(Direction::Down), context),
    ]);
}
