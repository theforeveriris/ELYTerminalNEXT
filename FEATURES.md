# Feature gap analysis

A survey of what a complete terminal emulator has, against what this fork has.

Sources: Terminal Trove's 2026 comparison (44 emulators), Ghostty's feature docs, WezTerm's feature list, the kitty graphics protocol spec, and the upstream PRD's own `Deferred` section.

## What this fork already has

Stronger than its size suggests. These are all present and tested:

| Area | Notes |
|---|---|
| Terminal engine | `alacritty_terminal` 0.26, own pty, own OSC tap |
| Keyboard | xterm encoding, kitty keyboard protocol, win32-input-mode on Windows, IME composition |
| Mouse | X10/normal/button/any tracking, SGR + UTF-8, wheel, click-to-move-cursor |
| Shell integration | zsh, bash, fish, PowerShell; OSC 7 cwd, OSC 133 prompt marks |
| Program status | OSC 7501 rev 0.4 records, OSC 9;4 progress, herdr agent states |
| Notifications | OSC 9 / 777 / 99, system notifications, rate-limited |
| Hyperlinks | OSC 8, URL detection, herdr integration |
| Graphics (text) | box drawing, block elements, powerline as geometry |
| Find | regex-capable, searches scrollback, combined marks, wrapped lines |
| Layout | sessions column, terminal column, Git/Files column, draggable widths |
| Chrome | settings window, 14 themes, menus, blur on 4 platforms, system notifications |
| Personality | Ely, the animated mascot, with rig, moods and per-platform window glue |

## Gaps

Ordered by how much they cost a user per day.

### Tier 1 — most terminals have these, and their absence is felt

1. **Tabs.** No tab bar; sessions live in a left column only. 33 of 44 emulators have tabs.
   23 of 44 have split panes, 11 of 44 have a drop-down quick terminal.

2. **Split panes.** A workspace with one terminal per pane, so `tail -f` beside an editor needs no tmux.

3. **Key rebinding.** Every key is hardcoded in `bind_keys` calls. A user who wants Ctrl-Shift-S for split cannot have it.

4. **Configuration file.** Settings live only in `settings.json`, written by the settings window. No import/export, no per-profile config, no hot reload of a file the user edits.

5. **Session restore.** Closing the app loses every session and its working directory. Terminals that support it: Konsole, Tabby, Tilix, iTerm2, WezTerm.

### Tier 2 — real but less daily

6. **Image protocols.** No kitty graphics, no Sixel, no iTerm2 inline images. 22 of 44 emulators render images; `ls`, `catimg`, `chafa`, `kitty +kitten icat` all break.

7. **Reflow on resize.** Alacritty's grid does not reflow wrapped lines when the width changes, so narrowing the window loses the visual order of a long wrapped line.

8. **Synchronized output (DECSET 2026).** 14 of 44 emulators support it; without it, a fast program paints in visible steps.

9. **Profiles.** A named shell, working directory, environment and colour set, switchable.

10. **Scrollback search depth and options.** Find works, but there is no case toggle, no whole-word, no regex toggle, no search-in-selection.

11. **Click to run a command.** Shell integration knows the command boundaries; a click could rerun it. The data is already there from OSC 133.

### Tier 3 — polish

12. **Quick terminal / drop-down.** 11 of 44 have one.
13. **Grapheme cluster correctness.** Multi-codepoint emoji, flags, skin tones, combining marks.
14. **Right-to-left scripts.** Arabic and Hebrew shaping.
15. **Configurable margins and cursor style per profile.**
16. **Startup / shell-directory default beyond `StartIn`.**
17. **Plugin or scripting layer.** WezTerm has Lua; Hyper has JS.
18. **Clipboard history.**
19. **Broadcast input to all panes.**
20. **Pane-level zoom.**

## Deliberately out of scope for this fork

Per the fork's README, this is a personal adaptation maintained by an AI agent, iterating aggressively, where the maintainer's needs outrank the general audience. Features nobody in this fork's one-person setup will use are not worth the maintenance. So the criteria below are deliberately narrow.

## Suggested order

1. **Split panes and tabs** — the largest daily gap, and the one that most changes what the app feels like. Pane focus movement, pane resize, close.
2. **Key rebinding through the settings window** — small, and it unlocks the pane shortcuts from step 1.
3. **Session restore** — save sessions and their working directories, restore on start.
4. **Reflow on resize** — correctness fix in the grid layer.
5. **Kitty graphics** — the largest protocol addition, but the payoff is visible: inline images work.
6. **Profiles and a configuration file** — a base for the above, and for anything the maintainer invents later.
7. **Click to run a command** — cheap, because OSC 133 already marks the boundaries.
