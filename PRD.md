# ELYTerminal PRD
# ELYTerminal 产品需求文档

A terminal for macOS, Windows and Linux. Rust, GPUI, Ely components. It draws a terminal as well as Ghostty does, shows every setting on screen, and makes no noise.

一个面向 macOS、Windows 与 Linux 的终端。用 Rust 编写，基于 GPUI 与 Ely 组件。它把终端画得和 Ghostty 一样好，把每一项设置都呈现在界面上，并且不发出任何声音。

Chapters run in build order. `TASKS.md` follows them line by line.

各章按构建顺序推进。`TASKS.md` 逐行对应这些章节。

## 1. Product
## 1. 产品

- One window. Sessions on the left, the terminal in the middle, Git or Files on the right.

  单一窗口。会话在左，终端在中，Git 或文件在右。

- Every setting has a control in the settings window. No setting lives only in a file.

  每一项设置在设置窗口中都有对应控件。没有任何设置只存在于文件里。

- Silent: no sound, no bell, no notifications but a session's, below, no update checks, no telemetry, no tips, no onboarding.

  静默：没有声音，没有响铃，除了下文会话自身的通知外没有别的通知，没有更新检查，没有遥测，没有提示语，没有引导流程。

- Clean: no subtitle, no tagline, no logging options, no status bar, no badges that do not carry state.

  简洁：没有副标题，没有标语，没有日志选项，没有状态栏，没有不承载状态的角标。

- Built-in themes, each shown as a small preview, Catppuccin among them. A theme colors the terminal and the chrome.

  内置主题，各以小预览展示，其中包括 Catppuccin。主题同时为终端与界面框架着色。

- The window background blurs on macOS, Windows, Wayland and X11.

  窗口背景在 macOS、Windows、Wayland 与 X11 上会模糊。

## 2. Foundation
## 2. 基础

- One binary crate, `elyterminal`, edition 2024, Rust 1.95.

  单一二进制 crate `elyterminal`，edition 2024，Rust 1.95。

- Ely from github.com/ELY-Open-Source by pinned rev. gpui, gpui_platform from crates.io `gpui-pre` at the exact version Ely pins.

  Ely 来自 github.com/ELY-Open-Source，按固定 rev 引用；gpui 与 gpui_platform 来自 crates.io 的 `gpui-pre`，版本与 Ely 所固定的完全一致。

- `alacritty_terminal` 0.26 for the grid, the parser and the pseudo-terminal, as Ely uses it.

  使用 `alacritty_terminal` 0.26 承担网格、解析器与伪终端，与 Ely 的用法一致。

- Logs through `log` to stderr, `RUST_LOG` sets the level. No log setting in the app.

  通过 `log` 将日志写到 stderr，由 `RUST_LOG` 设定级别。应用内不设日志选项。

- CI on GitHub Actions builds the Windows executable only, on a self-hosted runner: the zip from `scripts/windows-zip.ps1`, with the icon in the executable.

  GitHub Actions 上的 CI 只构建 Windows 可执行文件，使用自托管 runner：由 `scripts/windows-zip.ps1` 产出 zip，图标已嵌入可执行文件内。

- Repo `theforeveriris/ELYTerminalNEXT` on GitHub, public. No credential in any file or commit.

  仓库 `theforeveriris/ELYTerminalNEXT` 位于 GitHub，公开。任何文件与提交中都没有凭据。

## 3. Settings model and storage
## 3. 设置模型与存储

- One JSON file, `settings.json`, in the platform config folder (`dirs::config_dir()/ELYTerminal`).

  单个 JSON 文件 `settings.json`，位于平台配置目录（`dirs::config_dir()/ELYTerminal`）。

- Every field has a default. A missing file means defaults. A missing field means its default.

  每个字段都有默认值。文件缺失即使用默认值，字段缺失即使用该字段的默认值。

- A file that does not parse, or a value out of range, stops the start with an error naming the file and the field. The app never overwrites a file it could not read.

  若文件无法解析，或其中含有超出范围的取值，启动会中止并给出指明文件与字段的错误。应用绝不会覆写它读不到的文件。

- Writes go to a temp file in the same folder, then rename. One writer: the app's settings entity.

  写入先落到同目录的临时文件，再改名。写入方唯一：应用的设置实体。

- Fields: see chapter 10.

  字段：见第 10 章。

## 4. Themes
## 4. 主题

- Built in: Ely Dark, Ely Light, Catppuccin Mocha, Catppuccin Macchiato, Catppuccin Frappé, Catppuccin Latte, Tokyo Night, Dracula, Nord, Gruvbox Dark, Rosé Pine, Solarized Light, One Light, Kanagawa Wave.

  内置：Ely Dark、Ely Light、Catppuccin Mocha、Catppuccin Macchiato、Catppuccin Frappé、Catppuccin Latte、Tokyo Night、Dracula、Nord、Gruvbox Dark、Rosé Pine、Solarized Light、One Light、Kanagawa Wave。

- A theme holds background, foreground, cursor, cursor text, selection background and foreground, and the 16 ANSI colors. Values come from the theme's upstream files (iTerm2-Color-Schemes' Ghostty export, or Ely's palette).

  一个主题包含背景、前景、光标、光标文本、选区背景与前景，以及 16 种 ANSI 颜色。取值来自主题的上游文件（iTerm2-Color-Schemes 的 Ghostty 导出，或 Ely 的调色板）。

- A theme maps onto Ely's `Palette` through `Theme::set_palette`, so panels, inputs and lists wear the theme.

  主题通过 `Theme::set_palette` 映射到 Ely 的 `Palette`，因此面板、输入框与列表都会穿上该主题。

- Dark mode: one theme, or a second theme for when the system is dark.

  深色模式：可以使用同一个主题，或指定第二个主题在系统处于深色时接管。

- On Omarchy 4 (Linux), an Omarchy theme wears Omarchy's current theme and changes with each switch. It is the default where Omarchy runs; picking another theme stops following.

  在 Omarchy 4（Linux）上，Omarchy 主题会穿上 Omarchy 当前的主题，并随每次切换而变化。它在运行 Omarchy 的环境中是默认主题；选择其他主题即停止跟随。

## 5. Terminal engine
## 5. 终端引擎

Ghostty is the bar for what the terminal does. ELYTerminal owns its engine on `alacritty_terminal`; Ely's `Terminal` is a gallery widget without mouse reporting, IME, kitty keys or style settings.

Ghostty 是终端能力的标杆。ELYTerminal 基于 `alacritty_terminal` 自持引擎；Ely 的 `Terminal` 是一个展示用控件，不支持鼠标上报、输入法、kitty 按键或样式设置。

- PTY: alacritty's `tty` and event loop. The pty is wrapped so its reader passes bytes through an OSC scanner. The scanner reports the working directory and prompt marks.

  PTY：使用 alacritty 的 `tty` 与事件循环。pty 被包装，其读取器会让字节先经过 OSC 扫描器。扫描器报告工作目录与提示符标记。

- Shell integration for zsh, bash, fish and PowerShell, injected at launch as Ghostty does: each prompt sends OSC 7 and a title. A shell without integration reports nothing, and the panels say so.

  面向 zsh、bash、fish 与 PowerShell 的 shell 集成，如 Ghostty 那样在启动时注入：每个提示符都发送 OSC 7 与标题。没有集成时 shell 不会报告任何内容，面板会如实说明。

- Prompt marks, OSC 133: the shell integration marks where each prompt starts (A) and ends (B), where a command starts (C), and where it ends with its exit status (D), all but C on macOS's bash 3.2, which has no hook for it; the terminal reads any program's OSC 133. Cmd-Up and Cmd-Down, and Cmd-Shift-Up and Cmd-Shift-Down, on macOS, Ctrl-Shift-Up and Ctrl-Shift-Down elsewhere, scroll to the prompt before or after the view's top, as in Ghostty.

  提示符标记，OSC 133：shell 集成标记每个提示符的起点（A）与终点（B）、每条命令的起点（C）及其带退出状态的终点（D）；macOS 的 bash 3.2 没有相应钩子，故除 C 之外均不标记；终端读取任何程序发出的 OSC 133。macOS 上用 Cmd-上、Cmd-下及 Cmd-Shift-上、Cmd-Shift-下，其他平台用 Ctrl-Shift-上、Ctrl-Shift-下，像 Ghostty 那样滚动到视图顶部之前或之后的提示符。

- Program status, OSC 7501 revision 0.4: each terminal keeps the records programs report (state, id, kind, progress, app, title, msg), checked as the spec checks them; a new prompt drops working, blocked and idle records, a full reset drops them all; the terminal answers the support query with the states and kinds it takes.

  程序状态，OSC 7501 修订版 0.4：每个终端保存程序报告的记录（状态、id、类型、进度、应用、标题、消息），其校验方式与规范一致；新提示符会丢弃 working、blocked 与 idle 记录，完全重置会丢弃全部记录；终端以它所支持的状态与类型回应支持查询。

- Progress, OSC 9;4: set, error, indeterminate and pause stand as the root record until a program reports OSC 7501, until a full reset.

  进度，OSC 9;4：set、error、indeterminate 与 pause 作为根记录存在，直到程序报告 OSC 7501 或发生完全重置。

- Notifications: OSC 9 (iTerm2), OSC 777 notify (rxvt) and OSC 99 (kitty: title and body in chunks, base64, ids, focus and report on activation, close, expiry, which ones live, and its query; `o` passes by, since a session posts only while unseen, below); ConEmu's other OSC 9 commands are no notifications.

  通知：OSC 9（iTerm2）、OSC 777 notify（rxvt）与 OSC 99（kitty：分块的标题与正文、base64、id、激活时的聚焦与上报、关闭、过期、哪些仍然存活，以及它的查询；`o` 略过，因为会话只在未被查看时才发布通知，见下文）；ConEmu 的其他 OSC 9 命令不是通知。

- Inside herdr, which keeps its panes' OSC 7, the session's folder is that of herdr's focused pane, read from herdr's socket while herdr's client runs in the session, on macOS, Linux and Windows.

  在 herdr 内部，由于它保留了各面板的 OSC 7，会话的目录取自 herdr 的焦点面板；只要 herdr 的客户端在该会话中运行，就从 herdr 的套接字读取，支持 macOS、Linux 与 Windows。

- Inside herdr, which passes no OSC 7501 on, herdr's agent states (idle, working, blocked, done) are the session's status records.

  在 herdr 内部，由于它不转发 OSC 7501，herdr 的 agent 状态（idle、working、blocked、done）即为该会话的状态记录。

- Grid: colors (named, 256, truecolor, OSC 4/10/11/12), bold, dim, italic, underline (single, double, curly, dotted, dashed), strikethrough, inverse, hidden, wide characters, combining marks.

  网格：颜色（具名、256、真彩、OSC 4/10/11/12）、粗体、淡显、斜体、下划线（单线、双线、波浪、点线、虚线）、删除线、反显、隐藏、宽字符、组合字符。

- Box drawing (U+2500–U+257F), block elements (U+2580–U+259F) and powerline shapes (U+E0B0–U+E0B7) painted as geometry, so lines and segments join across cells.

  制表符（U+2500–U+257F）、块元素（U+2580–U+259F）与 powerline 形状（U+E0B0–U+E0B7）以几何方式绘制，因此线条与线段可跨单元格连接。

- Fonts: JetBrains Mono by default, any installed font by choice, and fallback families tried in order for a character it lacks, a Chinese font after a Latin one among them; Nerd Font icons the font lacks come from a bundled Nerd Font.

  字体：默认 JetBrains Mono，也可选择任意已安装字体；回退字体族按顺序用于主字体缺失的字符，其中可包含排在拉丁字体之后的中文字体；主字体缺失的 Nerd Font 图标来自内置的 Nerd Font。

- Cursor: block, bar, underline, hollow block; blinking per setting and per DECSCUSR; hollow when the window is not focused.

  光标：块状、竖线、下划线、空心块；依设置与 DECSCUSR 闪烁；窗口未聚焦时为空心。

- Keys: xterm encoding with modifiers, application cursor mode, Option as Alt on macOS, the kitty keyboard protocol as the program sets its flags; a key Shift makes a symbol of goes as its own key with Shift, the symbol its alternate, as Ghostty sends it, on macOS and Windows. On Windows keys go as win32-input-mode records while ConPTY asks, as Windows Terminal sends them. Keypad keys type their characters, as Ghostty's do with mode 1035 on by default.

  按键：带修饰键的 xterm 编码、应用光标模式、macOS 上 Option 视作 Alt、kitty 键盘协议随程序设置的标志位生效；被 Shift 变为符号的按键按其自身键位连同 Shift 发出，该符号作为其备用键，与 Ghostty 在 macOS 与 Windows 上的发送方式一致。在 Windows 上，只要 ConPTY 请求，按键就以 win32-input-mode 记录发送，与 Windows Terminal 一致。小键盘按键直接键入其字符，与 Ghostty 默认开启模式 1035 时相同。

- IME: marked text drawn at the cursor; committed text sent to the program.

  输入法：组合中的文字绘制在光标处；提交的文字发送给程序。

- Mouse: select by character, word (double press) and line (triple press); Shift extends; mouse reporting (X10, normal, button, any; SGR and UTF-8); wheel scrolls back, or reports, or sends arrows on the alternate screen.

  鼠标：按字符选区、按单词（双击）与按行（三击）选择；Shift 扩展选区；鼠标上报（X10、normal、button、any；SGR 与 UTF-8）；滚轮回滚历史，或上报给程序，或在备用屏上发送方向键。

- Click to move: while the shell reads a line, as its OSC 133 marks say, a plain click on that line, past its prompt, moves the shell's cursor there, as Ghostty's `cursor-click-to-move`: a shell that takes clicks, as fish does, hears the click; others get arrow keys.

  点击移动：在 shell 正在读取某一行时（依其 OSC 133 标记判断），对该行提示符之后的普通点击会把 shell 的光标移到该处，与 Ghostty 的 `cursor-click-to-move` 一致：像 fish 那样接受点击的 shell 会收到该点击；其他 shell 则收到方向键。

- Clipboard: copy, paste (bracketed when asked), copy on select per setting, OSC 52 writes.

  剪贴板：复制、粘贴（被请求时带括号）、依设置的选中即复制、OSC 52 写入。

- Drop: files dropped on the terminal type their paths at the cursor, as macOS Terminal does, each kept literal for the session's shell and followed by a space.

  拖放：拖到终端上的文件会在光标处键入其路径，与 macOS Terminal 一致，每个路径对会话的 shell 保持字面量形式并以空格结尾。

- Links: OSC 8 and plain URLs; Cmd-press (Ctrl-press off macOS) opens them. Inside herdr, while it takes the mouse, herdr opens them, web links only, as herdr does, whole where its pane wraps them, and underlines them while the key is held; with Shift the terminal opens what it sees.

  链接：OSC 8 与普通 URL；按住 Cmd 点击（在 macOS 之外为按住 Ctrl 点击）打开它们。在 herdr 内部，只要它接管鼠标，链接就由 herdr 打开，仅限网页链接，与 herdr 自身一致，其面板换行处按整段打开，并在按住按键期间为其加下划线；按住 Shift 则由终端打开它所看到的链接。

- Focus reporting, title (OSC 0/2), scrollback limit per setting, find in scrollback (Ely's `FindWidget`).

  焦点上报、标题（OSC 0/2）、依设置限制的回滚长度、在回滚中查找（Ely 的 `FindWidget`）。

- A scrollbar over the history, along the terminal's right edge: a drag on its thumb scrolls, whatever the program does with the wheel.

  沿终端右缘覆盖在历史之上的滚动条：拖动其滑块即可滚动，无论程序如何处理滚轮。

- Bell: no sound. The session row marks it until the session is shown.

  响铃：不发声。会话行会加以标记，直到该会话被显示。

## 6. Main window and layout
## 6. 主窗口与布局

- Ely's `ColumnShell`: sessions column, terminal column, right column. The window opens with blur when the setting is on.

  Ely 的 `ColumnShell`：会话栏、终端栏、右侧栏。开启该设置时窗口以模糊效果打开。

- The sessions column and the right column toggle, each by a button at its end of the title bar or by key, and keep their state in settings, so the window opens as it was left.

  会话栏与右侧栏各自可切换，方式为标题栏对应端的按钮或快捷键，其状态保存在设置中，因此窗口按上次离开时的样子打开。

- The sessions column and the right column resize by a drag on their border toward the terminal, on macOS, Windows and Linux, from 160 to 960 pixels; the widths are kept in settings once a drag rests, so the window opens with them.

  会话栏与右侧栏可在 macOS、Windows 与 Linux 上，通过朝终端方向拖动其边框来调整宽度，范围 160 到 960 像素；拖动停止后宽度保存到设置中，因此窗口按此打开。

- The window opens at the size it was left at, on macOS, Windows and Linux: the size is kept in settings once it settles; a maximized or full-screen window keeps the size from before; a kept size past the screen shrinks to it; the window is no smaller than 100 by 100, the least the settings keep.

  窗口在 macOS、Windows 与 Linux 上按上次离开时的尺寸打开：尺寸稳定后保存到设置中；最大化或全屏的窗口保留此前的尺寸；超出屏幕的保存尺寸会收缩至屏幕范围；窗口不小于 100 乘 100，这是设置所保留的最小值。

- The title shows the active session's title, and no other text.

  标题显示当前会话的标题，不含其他文字。

- The pointer over the terminal is the arrow.

  终端之上的指针为箭头。

- Ely, the icon's girl, peeks over the window's top edge from behind it, on macOS, Windows and Linux; where no room lies above the window, at the screen's top or full screen, she stands inside it at its bottom left. She moves all the time, as she does in the films: she breathes, sways and blinks, and how the sessions are doing, waiting, an error, work, a finish or calm, moves her. Nothing but her shows: no frame, line or shadow around her. A hop lifts more of her from behind the edge, never her body off it. A press pets her, and wakes her from a doze. A drag moves her along the edge she stands on, and she stays where she is let go. A switch in Settings, Terminal, turns her off.

  Ely，即图标中的女孩，在 macOS、Windows 与 Linux 上从窗口背后探出上缘；当窗口上方没有空间时（位于屏幕顶部或全屏），她改为站在窗口内部的左下角。她始终在动，如同影片中一样：呼吸、摇摆、眨眼，而会话的状态——等待、报错、工作、完成或平静——会驱动她。她之外不显示任何东西：没有边框、线条或阴影。跳跃会让她从边缘后方露出更多，而绝不会让整个身体离开边缘。按下会抚她，并从瞌睡中唤醒她。拖动可沿她所站立的边缘移动她，松手后她便留在那里。设置 › Terminal 中的开关可关掉她。

- The title bar is transparent, as Ghostty's: the window's background, opacity and blur run under it, so it wears the terminal's color.

  标题栏如 Ghostty 那样透明：窗口的背景、不透明度与模糊延伸至其下，因此它呈现出终端的颜色。

- Every control in the title bar takes its own press on every platform, the window buttons Ely draws on Windows and Linux too.

  标题栏中的每个控件在所有平台上都能各自响应点击，Ely 在 Windows 与 Linux 上绘制的窗口按钮同样如此。

## 7. Sessions panel
## 7. 会话面板

- A list of sessions. Each row: title, then the working directory, home as `~`.

  会话列表。每一行：标题，然后是工作目录，home 显示为 `~`。

- New session (+ and Cmd-T), close (row's ×, Cmd-W), select (press, Cmd-1…9, Ctrl-Tab and Ctrl-Shift-Tab).

  新建会话（+ 与 Cmd-T）、关闭（行上的 ×、Cmd-W）、选择（点击、Cmd-1…9、Ctrl-Tab 与 Ctrl-Shift-Tab）。

- Rename: a double press on a row's title opens a field in its place; Enter keeps the name, Escape drops it. The name shows in the row, the terminal's header and the window's title over the program's own until renamed again; an empty name gives the title back to the program.

  重命名：双击行上的标题会在原处打开一个输入框；Enter 保留名称，Escape 放弃。名称显示在行上、终端的标题栏以及窗口标题中，压过程序自身的标题，直到再次重命名；名称为空则把标题交还给程序。

- A session whose program ends closes its row. The last row closing closes the window.

  程序结束的会话会关闭其所在行。最后一行关闭时窗口也随之关闭。

- A bell in a hidden session marks its row.

  隐藏会话中的响铃会标记其所在行。

- A row shows its session's status: working, with its progress where known; waiting on the user, for a permission, a question or a login; done; error; and a mark when the last command failed. Hover shows each record's program, title and message. A hidden session that starts waiting marks its row until it is shown.

  每一行显示其会话的状态：工作中，若已知则附带进度；等待用户（等待授权、提问或登录）；完成；报错；以及上一条命令失败时的标记。悬停会显示每条记录的程序、标题与消息。开始等待的隐藏会话会标记其所在行，直到被显示。

- The mark is 3 by 3 dots that move: work ripples, or fills by its progress; waiting dips; a settled shape draws its shape, then holds still.

  标记是会动的 3 乘 3 点阵：工作时泛起涟漪，或按进度填充；等待时起伏下沉；已定型的状态先画出其形状，随后静止。

- System notifications on macOS, Windows and Linux: a session that starts waiting, finishes or fails, and a program's OSC 9, 777 or 99 notification, post one while the window is not active or the session is hidden, one a session every 3 seconds at most; it names the session, and a press on it shows the window and the session. On macOS only the app bundle posts them.

  macOS、Windows 与 Linux 上的系统通知：开始等待、完成或失败的会话，以及程序通过 OSC 9、777 或 99 发出的通知，在窗口非活跃或会话被隐藏时各发布一条，每个会话最多每 3 秒一条；通知标明明会话，按下会显示窗口与该会话。在 macOS 上只有 app bundle 会发布通知。

## 8. Right panel: Files
## 8. 右侧面板：文件

- A switch at the top, centered below the title bar: Git, Files.

  顶部的切换控件，居中于标题栏之下：Git、文件。

- Files shows the active session's working directory as a tree, folders first, Git status colors on names.

  文件面板以树形展示当前会话的工作目录，文件夹在前，名称上带有 Git 状态颜色。

- Double press: a folder opens, a file opens in the system's default app.

  双击：文件夹打开，文件用系统默认应用打开。

- It reloads when the session reports a new directory and at each prompt.

  当会话报告了新目录以及在每个提示符处，它会重新加载。

## 9. Right panel: Git
## 9. 右侧面板：Git

- Through the `git` command line. No libgit2.

  通过 `git` 命令行。不使用 libgit2。

- Shows the branch, staged and unstaged changes, a commit message field and Commit, and recent commits.

  显示分支、已暂存与未暂存的改动、提交信息输入框与 Commit 按钮，以及最近的提交。

- Stage and unstage a file. Press a file to see its diff in Ely's `DiffViewer`.

  暂存与取消暂存文件。点击文件可在 Ely 的 `DiffViewer` 中查看其差异。

- Discard, which Ely's changes list draws on every unstaged and untracked row, asks first: an edit goes back to the index, a new file is deleted.

  丢弃（Ely 的改动列表会在每个未暂存与未跟踪行上绘出该操作）会先询问：已修改的编辑退回索引，新增文件被删除。

- Outside a repository it says so. A failed git command shows its error.

  不在仓库中时会如实说明。git 命令失败时显示其错误。

- It reloads at each prompt and on directory change.

  它在每个提示符处以及目录变化时重新加载。

## 10. Settings window
## 10. 设置窗口

Cmd-, opens it; on Windows and Linux, which have no menu bar, so does a gear in the title bar. Categories on the left, rows on the right, each side scrolling on its own, Ely's components. Changes apply at once to every session and save.

Cmd-, 打开设置窗口；在没有菜单栏的 Windows 与 Linux 上，标题栏中的齿轮同样可以打开。左侧为分类，右侧为条目，两侧各自独立滚动，均使用 Ely 的组件。改动会立即作用于每个会话并保存。

- Appearance
- 外观
  - Theme: a grid of previews; on Omarchy, the Omarchy theme first.

    主题：预览网格；在 Omarchy 上，Omarchy 主题排在最前。

  - Separate dark theme: a switch; when on, a second grid for dark mode.

    单独的深色主题：一个开关；开启后出现第二个用于深色模式的网格。

  - Font family: a list of installed fonts. Fallbacks: installed families tried in order for a character the family lacks, each added, changed or removed in a list of pickers. Font size. Line height. Ligatures.

    字体族：已安装字体的列表。回退字体：在主字体族缺失某字符时按顺序尝试的已安装字体族，每项都可在选择器列表中添加、更改或移除。字号。行高。连字。

  - Cursor: style (block, bar, underline, hollow), blink, with a live preview.

    光标：样式（块状、竖线、下划线、空心）、闪烁，附实时预览。

  - Background opacity: a slider. Background blur: a switch.

    背景不透明度：滑块。背景模糊：开关。

  - Padding: horizontal and vertical.

    内边距：水平与垂直。
- Terminal
- 终端
  - Shell: a path; empty runs the login shell.

    Shell：一条路径；留空则运行登录 shell。

  - Start in: Home, or the active session's folder.

    起始目录：Home，或当前会话所在目录。

  - Scrollback lines.

    回滚行数。

  - Option as Alt: off, left, right, both (macOS only).

    Option 视作 Alt：关闭、左侧、右侧、两侧（仅 macOS）。

  - Copy on select.

    选中即复制。

  - Ely: a switch.

    Ely：开关。

  - Check for updates automatically: a switch, on by default, which checks at start and once a day.

    自动检查更新：开关，默认开启，在启动时及每天检查一次。

  - Updates: what the last check found, and Check now; a newer release brings Download, which opens its page, and one an automatic check finds posts a notification.

    更新：上次检查的结果，以及 Check now 按钮；若发现新版本会显示 Download 并打开其页面，自动检查发现新版本时还会发布一条通知。

  - Shortcuts: every shortcut, as this platform's keys, with a search.

    快捷键：全部快捷键，按本平台的键位列出，并可搜索。

## 11. Background blur
## 11. 背景模糊

- macOS: gpui's `WindowBackgroundAppearance::Blurred` (NSVisualEffectView).

  macOS：gpui 的 `WindowBackgroundAppearance::Blurred`（NSVisualEffectView）。

- Windows: gpui's `Blurred` (acrylic accent). On Windows 10, whose DWM drags an acrylic window seconds behind the pointer, the glass clears while the window moves or sizes and blurs again when it rests; Windows 11 keeps its acrylic throughout.

  Windows：gpui 的 `Blurred`（acrylic 强调色）。在 Windows 10 上，其 DWM 会把带 acrylic 的窗口拖在指针后数秒，因此窗口移动或缩放时玻璃会变为透明，停稳后重新模糊；Windows 11 全程保留其 acrylic。

- Wayland: gpui's `Blurred` speaks `org_kde_kwin_blur`; ELYTerminal adds `ext_background_effect_v1` on the window's surface, which GNOME 51, Plasma 6.7 and Hyprland support. Hyprland honors it only while its own blur is on, which Omarchy 4.0.4 turns off.

  Wayland：gpui 的 `Blurred` 使用 `org_kde_kwin_blur`；ELYTerminal 另在窗口表面上加上 `ext_background_effect_v1`，GNOME 51、Plasma 6.7 与 Hyprland 均支持。Hyprland 仅在其自身模糊开启时才认这项协议，而 Omarchy 4.0.4 将其关闭。

- X11: ELYTerminal sets `_KDE_NET_WM_BLUR_BEHIND_REGION` (CARDINAL, 32) on the window, as Ghostty does; removes it when blur turns off.

  X11：ELYTerminal 如 Ghostty 那样在窗口上设置 `_KDE_NET_WM_BLUR_BEHIND_REGION`（CARDINAL，32）；关闭模糊时将其移除。

- A compositor with neither protocol shows transparency without blur. The app logs which path it took.

  两种协议都不支持的合成器只显示透明而没有模糊。应用会记录它走了哪条路径。

## 12. Menus and shortcuts
## 12. 菜单与快捷键

- macOS menu bar: ELYTerminal (Settings, Quit), Shell (New Session, Close Session), Edit (Copy, Paste, Select All, Find), View (Toggle Sessions, Toggle Right Panel, Bigger, Smaller, Reset Size), Window.

  macOS 菜单栏：ELYTerminal（Settings、Quit）、Shell（New Session、Close Session）、Edit（Copy、Paste、Select All、Find）、View（Toggle Sessions、Toggle Right Panel、Bigger、Smaller、Reset Size）、Window。

- The same shortcuts on Windows and Linux with Ctrl-Shift for terminal-colliding keys. On Windows Ctrl-V pastes too, as in Windows Terminal.

  Windows 与 Linux 上使用相同的快捷键，其中与终端冲突的按键用 Ctrl-Shift。在 Windows 上 Ctrl-V 也可粘贴，与 Windows Terminal 一致。

## 13. Acceptance
## 13. 验收

- Each chapter's tasks ticked in `TASKS.md` with a passing review in `tasks/reviews.md`.

  各章的任务在 `TASKS.md` 中勾选，并在 `tasks/reviews.md` 中有通过的评审。

- `scripts/check.sh` passes: fmt, clippy with `-D warnings`, tests, no file over 500 lines.

  `scripts/check.sh` 通过：fmt、带 `-D warnings` 的 clippy、测试，且没有超过 500 行的文件。

- CI green: the Windows executable builds on the self-hosted runner.

  CI 为绿：Windows 可执行文件在自托管 runner 上构建成功。

## 14. Icon and the macOS app
## 14. 图标与 macOS 应用

- The icon is Ely, a girl with long pink hair, holding a terminal in front of her.

  图标是 Ely，一位粉色长发的女孩，双手在身前捧着一个终端。

- macOS: the icon on the rounded square macOS gives app icons. Windows and Linux: the figure alone, no ground, flat, as Papirus draws.

  macOS：图标置于 macOS 为应用图标提供的圆角方块上。Windows 与 Linux：仅保留人物本身，无底色，扁平绘制，与 Papirus 的画法一致。

- `scripts/mac-app.sh` builds `ELYTerminal.app` from a release build with its icon, signed ad hoc, or for a release with Developer ID and notarized, and a disk image holding it beside a link to Applications.

  `scripts/mac-app.sh` 从带图标的发行版构建产出 `ELYTerminal.app`，本地构建时临时签名，发行时以 Developer ID 签名并公证，同时生成一个内含该 app 的磁盘映像，旁边附有指向应用程序的链接。

## 15. Release
## 15. 发布

- A release on GitHub, tagged `vVERSION`, carries the Windows zip (x86_64).

  GitHub 上以 `vVERSION` 标记的发布，附带 Windows zip（x86_64）。

- The Windows executable carries the icon. The window names itself after it, `com.zacharyzhang.elyterminal`, as the macOS bundle does.

  Windows 可执行文件内含图标。窗口以 `com.zacharyzhang.elyterminal` 命名自身，与 macOS bundle 相同。

- The build comes from `scripts/windows-zip.ps1`, on the self-hosted Windows runner.

  构建由 `scripts/windows-zip.ps1` 在自托管的 Windows runner 上完成。

## Deferred
## 暂缓

Splits, tabs inside a session, profiles, key rebinding, kitty graphics, sixel, session restore, push and pull, quick terminal.

分屏、会话内标签页、配置文件（profile）、按键重绑定、kitty 图形、sixel、会话恢复、推送与拉取、快捷终端。
