# Improvement plan

The fork's gaps, in the order they get built. See `FEATURES.md` for the survey behind them.

## Milestone 1 — Tabs and split panes

The largest daily gap: 33 of 44 emulators have tabs, 23 of 44 have split panes.

### Model

The workspace's flat `sessions: Vec<Session>` becomes a tree:

```
Workspace
  tabs: Vec<Tab>          each tab is a page of the window
    panes: Vec<Pane>      each pane holds one terminal
      session: Session    the terminal and its row
```

`active_tab` and `active_pane` replace `active`. A session belongs to exactly one pane of exactly one tab, so the existing session list, the rows, the status marks and Ely's moods all keep working: they read the sessions, not the layout.

### Tabs

- A tab bar across the top of the terminal column, under the title bar.
- New tab, close tab, next/previous tab, and go to tab N.
- A tab's title is the focused pane's session title; a session renamed inside a tab renames the tab.
- The window title follows the focused pane, as it follows the focused session today.
- Tabs keep their own panes, so switching tabs does not spawn or kill anything.

### Panes

- Split the focused pane horizontally or vertically.
- Move focus between panes with the arrow keys, in a spatial order.
- Resize a pane by dragging the divider between it and its neighbour.
- Close a pane; the tab closes when its last pane goes.
- A pane that is not focused is dimmed at the edges, so the focus is obvious.

### Keys

Following Ghostty, because the fork follows Ghostty elsewhere:

| | macOS | Windows and Linux |
|---|---|---|
| New tab | Cmd-T | Ctrl-Shift-T |
| Close tab | Cmd-W | Ctrl-Shift-W |
| Next, previous tab | Ctrl-Tab, Ctrl-Shift-Tab | same |
| Split vertical, horizontal | Cmd-D, Cmd-Shift-D | Ctrl-Shift-D, Ctrl-Shift-Alt-D |
| Move focus | Cmd-Alt-Arrow | Ctrl-Shift-Alt-Arrow |
| Close pane | Cmd-Alt-W | Ctrl-Shift-Alt-W |

The existing session keys move to the tab bar's meaning: Cmd-T now opens a tab, and a new session is Cmd-Shift-T within a tab.

## Milestone 2 — Key rebinding

Every key is hardcoded in `bind_keys` today. A rebinding layer reads the user's bindings and applies them over the defaults, so a user can move a split shortcut without a rebuild.

## Milestone 3 — Lua configuration

LuaJIT 5.1, through `mlua`, with a vendored build so it works on all three platforms.

- `~/.config/ELYTerminal/init.lua`, or the platform's config folder, read at start.
- The Lua API can call into the app: register a keybinding, open a split, set a theme.
- The settings window and the Lua file are two views of the same model. The file wins where they disagree, and the settings window shows what the file set, so nothing is silently overridden.
- A bad Lua file stops the start with an error naming the file and the line, as a bad settings file does.

## Milestone 4 — Session restore

Save each session's command and working directory, restore them on the next start, in tabs and panes as they were left.

## Milestone 5 — Reflow on resize

Alacritty's grid does not reflow wrapped lines when the width changes. Narrowing the window loses the visual order of a long wrapped line. This is a grid-layer fix.

## Milestone 6 — Kitty graphics

The largest protocol addition, and the most visible: inline images from `catimg`, `chafa` and `kitty +kitten icat` would render.

The protocol is an APC, `ESC _ G key=value,... ; payload ESC \`, with the payload base64 in up to 4096-byte chunks, and three pixel formats: 24-bit RGB, 32-bit RGBA and PNG. Images place either in a cell (`U=1`) or over the text (`U=2`), and the terminal must hold the image until it is told to delete it.

## Milestone 7 — Profiles and a configuration file

A named shell, working directory, environment and colour set, switchable, with the Lua file as its base.

## Milestone 8 — Click to run a command

Shell integration already marks command boundaries with OSC 133, so the data is there. A click on a command line runs it again.
