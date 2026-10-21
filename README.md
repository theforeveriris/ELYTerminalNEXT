# ELYTerminalNEXT
# ELYTerminalNEXT

A personal fork of [ELYTerminal](https://github.com/ELY-Open-Source/ELYTerminal), the terminal for macOS, Windows and Linux, in Rust on GPUI and Ely components. Sessions sit on the left, the terminal in the middle, Git or Files on the right. Every setting has a control in the settings window.

[ELYTerminal](https://github.com/ELY-Open-Source/ELYTerminal) 的个人二开版本。这是一个用 Rust 编写、基于 GPUI 与 Ely 组件的终端，支持 macOS、Windows 与 Linux：会话列表在左侧，终端在中间，Git 或文件面板在右侧。每一项设置都能在设置窗口里找到对应的控件。

## This is a fork
## 这是二开版本

ELYTerminalNEXT is a downstream, personal adaptation of ELYTerminal, carried out under upstream's MIT license, which allows it. It exists to fit one person's setup, not to replace the original.

ELYTerminalNEXT 是 ELYTerminal 的下游个人适配版，依上游的 MIT 许可协议进行，上游许可允许这样做。它的存在是为了贴合一个人的使用环境，而非取代原项目。

- **How it is maintained.** Mostly by AI, with a little human work: an AI agent reads the upstream code, makes the changes, and keeps the tree building and its tests green. Human involvement is limited to deciding what to adapt and reviewing the result.

  **维护方式。** 主要由 AI 维护，配合少量人工：AI agent 读取上游代码、做出修改，并让代码树保持可编译、测试保持全绿。人工的参与仅限于决定要适配什么，以及复核结果。

- **How it iterates.** More aggressively than upstream. Changes land as soon as they work, and behaviour that upstream would consider a compatibility question is fair game here, since the target is one user's machine rather than a general audience.

  **迭代节奏。** 比上游更激进。只要能跑通，改动就合入；上游会当作兼容性问题讨论的行为，在这里可以直接改动，因为目标是面向一台机器，而不是面向所有用户。

- **What that means for stability.** This fork favours what the maintainer wants over what every user needs. Behaviour may change between builds, and a change that upstream would treat as a regression may land here on purpose. If you want a stable terminal, use [upstream ELYTerminal](https://github.com/ELY-Open-Source/ELYTerminal) — it is the maintained, released and signed build, and it is the better choice.

  **对稳定性的影响。** 这个二开版本优先满足维护者的需求，而非全体用户的需要。行为可能随版本变化，上游会当作回归处理的改动，在这里可能是有意为之。**如果你需要功能稳定的终端，请选择上游的 [ELYTerminal](https://github.com/ELY-Open-Source/ELYTerminal)**——那才是持续维护、有正式发布且经过签名的版本，也是更好的选择。

Upstream is ELY-Open-Source/ELYTerminal; this fork keeps its licence (MIT, in `LICENSE`) and its credits, including the Ghostty-derived parts under `LICENSE-GHOSTTY`.

上游为 ELY-Open-Source/ELYTerminal；本二开版本保留其许可协议（MIT，见 `LICENSE`）与致谢，其中源自 Ghostty 的部分适用 `LICENSE-GHOSTTY`。

## Install
## 安装

This fork publishes no signed release. Build it yourself, or take upstream's [releases](https://github.com/ELY-Open-Source/ELYTerminal/releases) if you want a stable, signed build.

本二开版本不发布签名的发行版。请自行构建；若需要稳定且已签名的版本，请取用上游的[发行版](https://github.com/ELY-Open-Source/ELYTerminal/releases)。

Upstream's releases carry three builds:

上游的发行版提供三种构建：

- macOS 11 or later, Apple silicon: open `ELYTerminal-VERSION.dmg` and drag ELYTerminal into Applications. The app is signed with Developer ID and notarized.

  macOS 11 及以上、Apple 芯片：打开 `ELYTerminal-VERSION.dmg`，把 ELYTerminal 拖入应用程序目录。该应用经过 Developer ID 签名并已公证。

- Windows 10 or later, x86_64: unzip `ELYTerminal-VERSION-windows-x86_64.zip` and run `elyterminal.exe`. It is unsigned; if SmartScreen stops it, press More info, then Run anyway.

  Windows 10 及以上、x86_64：解压 `ELYTerminal-VERSION-windows-x86_64.zip` 并运行 `elyterminal.exe`。它未签名；若被 SmartScreen 拦下，点击"更多信息"后选择"仍要运行"。

- Linux, x86_64, glibc 2.35 or later, with libxkbcommon, libxkbcommon-x11 and libxcb:

  Linux、x86_64、glibc 2.35 及以上，并需安装 libxkbcommon、libxkbcommon-x11 与 libxcb：

```sh
tar -xzf ELYTerminal-VERSION-linux-x86_64.tar.gz
cd ELYTerminal-VERSION-linux-x86_64
install -Dm755 elyterminal -t ~/.local/bin
install -Dm644 com.zacharyzhang.elyterminal.desktop -t ~/.local/share/applications
install -Dm644 com.zacharyzhang.elyterminal.svg -t ~/.local/share/icons/hicolor/scalable/apps
```

The desktop entry runs `elyterminal`, so `~/.local/bin` must be on `PATH`.

该桌面项运行 `elyterminal`，因此 `~/.local/bin` 必须在 `PATH` 中。

## Build and run
## 构建与运行

This is the fork's own build. Rust 1.95.0, which `rust-toolchain.toml` pins; rustup installs it on the first build.

这是本二开版本自己的构建方式。Rust 1.95.0，由 `rust-toolchain.toml` 固定；rustup 会在首次构建时自动安装。

```sh
cargo run --release
```

Each platform's script builds its release in `target/release`: on macOS, `scripts/mac-app.sh` builds `ELYTerminal.app`, with its icon, and a disk image of it; on Windows, `scripts/windows-zip.ps1` zips `elyterminal.exe`, its icon inside; on Linux, `scripts/linux-tar.sh` builds in Docker and packs the binary with its desktop entry and icon.

各平台的脚本会在 `target/release` 中产出发行版：macOS 上 `scripts/mac-app.sh` 构建带图标的 `ELYTerminal.app` 及其磁盘映像；Windows 上 `scripts/windows-zip.ps1` 把内含图标的 `elyterminal.exe` 打成 zip；Linux 上 `scripts/linux-tar.sh` 在 Docker 中构建，并把二进制、桌面项与图标打包。

On Linux, install the libraries gpui links first. On Debian and Ubuntu:

在 Linux 上需先安装 gpui 链接所需的库。在 Debian 与 Ubuntu 上：

```sh
sudo apt-get install pkg-config libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libxcb1-dev libx11-xcb-dev libfontconfig-dev libfreetype-dev
```

The app writes its log to stderr. `RUST_LOG=elyterminal=debug` writes more; `trace` logs each paint. A Windows release build opens no console: `elyterminal.exe 2> log.txt` keeps its log.

应用把日志写到 stderr。`RUST_LOG=elyterminal=debug` 会输出更多；`trace` 会记录每一次绘制。Windows 的发行版构建不打开控制台：`elyterminal.exe 2> log.txt` 可保留日志。

## What it does
## 功能

- The terminal runs on `alacritty_terminal`: 256 colors and truecolor, bold, dim, italic, five underline styles, wide characters and combining marks. Box drawing, block elements and powerline shapes are drawn as geometry, so lines join across cells. Keys follow xterm and the kitty keyboard protocol, and on Windows win32-input-mode, as Windows Terminal sends them, so herdr's shortcuts work; input methods compose at the cursor; programs can track the mouse; pastes are bracketed when a program asks; OSC 8 and plain links open with the link press below; inside herdr, herdr opens them, web links only, whole where its pane wraps them, and with Shift the terminal opens what it sees; find searches the scrollback.

  终端运行在 `alacritty_terminal` 之上：支持 256 色与真彩色、粗体、淡显、斜体、五种下划线样式、宽字符与组合字符。制表符、块元素与 powerline 形状以几何方式绘制，因此线条可跨单元格连接。按键遵循 xterm 与 kitty 键盘协议，在 Windows 上还遵循 win32-input-mode（与 Windows Terminal 发送方式一致），因此 herdr 的快捷键可用；输入法在光标处组合；程序可以跟踪鼠标；程序请求时粘贴会带括号；OSC 8 与普通链接由下文的按键方式打开；在 herdr 内部由 herdr 打开链接，仅限网页链接，其面板换行处按整段打开，而按住 Shift 则由终端打开它所看到的链接；查找会搜索回滚缓冲。

- A scrollbar along the terminal's right edge scrolls its history, even while a program takes the wheel.

  终端右缘的滚动条用于滚动历史记录，即使程序接管了滚轮也依然有效。

- Shell integration for zsh, bash, fish and PowerShell: each prompt reports its folder and title, and OSC 133 marks each prompt, each command and its exit status.

  面向 zsh、bash、fish 与 PowerShell 的 shell 集成：每个提示符都会报告所在目录与标题，OSC 133 标记每个提示符、每条命令及其退出状态。

- Ely peeks over the window's top edge, or up from its bottom left when the window leaves her no room above it, and moves all the time: she breathes, sways and blinks. She follows your sessions: she hops when a program waits on you, shakes at an error, cheers a finish, reads along while a program works, and dozes when all is calm for long. Press her and she winks, waking her from a doze. Drag her along the edge to move her. Settings › Terminal › Ely turns her off.

  Ely 会从窗口上缘探出头来；若窗口上方没有足够空间，她便从左下角向上现身。她始终在动：呼吸、摇摆、眨眼。她跟随你的会话：程序在等你时她会跳跃，出错时她会摇头，完成时她会欢呼，程序工作中她会跟着读，长时间平静时她会打瞌睡。按下她会眨眼，从瞌睡中醒来。沿边缘拖动可移动她。在设置 › Terminal › Ely 中可以关掉她。

- Sessions: new, close and switch. A bell marks a hidden session's row and makes no sound. A row's mark shows what its programs report through OSC 7501, OSC 9;4 or herdr: waiting on you, an error, work with its progress, done or idle, else a failed last command. The mark is 3 by 3 dots: work ripples and waiting dips while they last. Hover over the mark lists each report.

  会话：新建、关闭与切换。响铃会标记隐藏会话所在的那一行，且不发出声音。行上的标记显示其程序通过 OSC 7501、OSC 9;4 或 herdr 报告的状态：等待你、报错、带有进度的工作、完成或空闲，否则为上一条命令失败。标记是 3 乘 3 的点阵：工作中涟漪扩散，等待时下沉的起伏会持续。悬停在标记上会列出每一条报告。

- Notifications: while the window is in the background or the session hidden, a session that starts waiting, finishes or fails, and a program's OSC 9, 777 or 99, post a system notification, one a session every 3 seconds at most; a press shows the session. On macOS only the app bundle posts them, not `cargo run`.

  通知：当窗口在后台或会话被隐藏时，开始等待、完成或失败的会话，以及程序通过 OSC 9、777 或 99 发出的请求，会发布系统通知；每个会话最多每 3 秒一条；按下通知会显示该会话。在 macOS 上只有 app bundle 才会发布通知，`cargo run` 不会。

- Cmd-Up and Cmd-Down, Ctrl-Shift-Up and Ctrl-Shift-Down off macOS, jump to the prompt before or after the view's top.

  Cmd-上 与 Cmd-下（在 macOS 之外为 Ctrl-Shift-上 与 Ctrl-Shift-下），跳到视图顶部之前或之后的提示符。

- A button at each end of the title bar hides or shows its side column, and a drag on the border between a side column and the terminal resizes it; the window opens as it was left, at the size and widths it was left at. On Windows and Linux a gear beside the right one opens Settings.

  标题栏两端各有一个按钮，可隐藏或显示对应的侧栏；拖动侧栏与终端之间的边框可调整其宽度；窗口会按上次离开时的尺寸与各栏宽度打开。在 Windows 和 Linux 上，右侧按钮旁还有一个齿轮可打开设置。

- Files shows the session's folder as a tree with Git's marks. Inside herdr the folder is that of herdr's focused pane.

  文件面板以树形展示会话所在目录，并标出 Git 状态。在 herdr 内部，目录为 herdr 焦点面板所在目录。

- Git shows the branch, stages, unstages, discards, commits, lists recent commits and opens diffs, all through the `git` command line.

  Git 面板显示分支、暂存、取消暂存、丢弃、提交，列出最近的提交并打开差异，全部通过 `git` 命令行完成。

- Themes: Ely Dark, Ely Light, Catppuccin Mocha, Macchiato, Frappé and Latte, Tokyo Night, Dracula, Nord, Gruvbox Dark, Rosé Pine, Solarized Light, One Light and Kanagawa Wave. A second theme can take over while the system is dark. On Omarchy 4, an Omarchy theme follows the system's theme as it switches, and is the default there.

  主题：Ely Dark、Ely Light、Catppuccin Mocha、Macchiato、Frappé 与 Latte、Tokyo Night、Dracula、Nord、Gruvbox Dark、Rosé Pine、Solarized Light、One Light 与 Kanagawa Wave。可指定第二个主题在系统处于深色模式时接管。在 Omarchy 4 上，Omarchy 主题会随系统主题切换而变化，并且是那里的默认主题。

- Background opacity and blur: NSVisualEffectView on macOS, acrylic on Windows, `ext_background_effect_v1` or `org_kde_kwin_blur` on Wayland, `_KDE_NET_WM_BLUR_BEHIND_REGION` on X11, which KWin reads. Blur shows only while the opacity is under 100%. Hyprland blurs only while its own `decoration:blur:enabled` is on, and Omarchy 4.0.4 turns it off; to turn it on there, add `hl.config({ decoration = { blur = { enabled = true } } })` to `~/.config/hypr/looknfeel.lua`.

  背景不透明度与模糊：macOS 上用 NSVisualEffectView，Windows 上用 acrylic，Wayland 上用 `ext_background_effect_v1` 或 `org_kde_kwin_blur`，X11 上用 KWin 读取的 `_KDE_NET_WM_BLUR_BEHIND_REGION`。仅当不透明度低于 100% 时才显示模糊。Hyprland 仅在其自身的 `decoration:blur:enabled` 开启时才模糊，而 Omarchy 4.0.4 将其关闭；要在此环境中开启，请向 `~/.config/hypr/looknfeel.lua` 添加 `hl.config({ decoration = { blur = { enabled = true } } })`。

## Keys
## 快捷键

Settings › Terminal lists them all, as this platform binds them.

设置 › Terminal 会按当前平台的绑定列出全部快捷键。

| | macOS | Windows and Linux |
|---|---|---|
| | macOS | Windows 与 Linux |
| New session | Cmd-T | Ctrl-Shift-T |
| 新建会话 | Cmd-T | Ctrl-Shift-T |
| Close session | Cmd-W | Ctrl-Shift-W |
| 关闭会话 | Cmd-W | Ctrl-Shift-W |
| Session 1 to 8, last | Cmd-1 to Cmd-8, Cmd-9 | Alt-1 to Alt-8, Alt-9 |
| 第 1 至 8 个会话，以及最后一个 | Cmd-1 到 Cmd-8、Cmd-9 | Alt-1 到 Alt-8、Alt-9 |
| Next, previous session | Ctrl-Tab, Ctrl-Shift-Tab | Ctrl-Tab, Ctrl-Shift-Tab |
| 下一个、上一个会话 | Ctrl-Tab、Ctrl-Shift-Tab | Ctrl-Tab、Ctrl-Shift-Tab |
| Sessions column | Cmd-B | Ctrl-Shift-B |
| 会话栏 | Cmd-B | Ctrl-Shift-B |
| Right column | Cmd-Alt-B | Ctrl-Shift-Alt-B |
| 右侧栏 | Cmd-Alt-B | Ctrl-Shift-Alt-B |
| Copy, paste | Cmd-C, Cmd-V | Ctrl-Shift-C, Ctrl-Shift-V; on Windows Ctrl-V pastes too |
| 复制、粘贴 | Cmd-C、Cmd-V | Ctrl-Shift-C、Ctrl-Shift-V；在 Windows 上 Ctrl-V 也可粘贴 |
| Select all | Cmd-A | Ctrl-Shift-A |
| 全选 | Cmd-A | Ctrl-Shift-A |
| Find | Cmd-F | Ctrl-Shift-F |
| 查找 | Cmd-F | Ctrl-Shift-F |
| Bigger, smaller, reset size | Cmd-=, Cmd-minus, Cmd-0 | Ctrl-=, Ctrl-minus, Ctrl-0 |
| 放大、缩小、重置字号 | Cmd-=、Cmd-minus、Cmd-0 | Ctrl-=、Ctrl-minus、Ctrl-0 |
| Open a link | Cmd-press | Ctrl-press |
| 打开链接 | 按住 Cmd 点击 | 按住 Ctrl 点击 |
| Settings | Cmd-, | Ctrl-, |
| 设置 | Cmd-, | Ctrl-, |
| Minimize | Cmd-M | |
| 最小化 | Cmd-M | |
| Quit | Cmd-Q | Ctrl-Shift-Q |
| 退出 | Cmd-Q | Ctrl-Shift-Q |

## Settings
## 设置

The settings window saves each change at once to `settings.json` in the platform's config folder:

设置窗口会把每一次改动立即保存到平台配置目录下的 `settings.json`：

- macOS: `~/Library/Application Support/ELYTerminal/settings.json`

  macOS：`~/Library/Application Support/ELYTerminal/settings.json`

- Linux: `$XDG_CONFIG_HOME/ELYTerminal/settings.json`, else `~/.config/ELYTerminal/settings.json`

  Linux：`$XDG_CONFIG_HOME/ELYTerminal/settings.json`，否则为 `~/.config/ELYTerminal/settings.json`

- Windows: `%APPDATA%\ELYTerminal\settings.json`

  Windows：`%APPDATA%\ELYTerminal\settings.json`

A file that does not parse, or holds a value out of range, stops the start with an error naming the file and the field. The app never overwrites it.

若文件无法解析，或其中含有超出范围的取值，启动会中止并给出指明文件与字段的错误。应用绝不会覆写该文件。

Settings › Terminal checks for a newer release at start and once a day, or at once with Check now; a newer one shows a Download button that opens its page. The check runs `curl`, which macOS and Windows ship. It reads upstream's releases, so it reports upstream's versions, not this fork's.

设置 › Terminal 会在启动时以及每天检查一次是否有更新版本，也可通过 Check now 立即检查；若发现新版本，会显示一个 Download 按钮并打开其页面。该检查通过 `curl` 运行，macOS 与 Windows 均自带此工具。它读取的是上游的发行版，因此报告的是上游的版本号，而非本二开版本的。

## Development
## 开发

`scripts/check.sh` runs fmt, clippy, the tests and the 500-line rule. `PRD.md` says what the app does, and `TASKS.md` says how far it is.

`scripts/check.sh` 会运行 fmt、clippy、测试与 500 行规则。`PRD.md` 说明应用的功能，`TASKS.md` 说明其完成度。

Changes here are made by an AI agent and reviewed by a human. The bar for a change is that it works for this fork's maintainer, not that it would be accepted upstream: the tests and the check script must pass, and a change that breaks upstream's behaviour on purpose is a decision to record, not an accident.

此处的改动由 AI agent 完成、人工复核。对一项改动的要求是它对本二开版本的维护者有用，而非它会被上游接受：测试与检查脚本必须通过；刻意破坏上游行为的改动属于需要记录的决策，而非意外。

## License
## 许可

MIT, in `LICENSE`, as upstream ELYTerminal's, which this fork keeps and whose copyright it carries. Box drawing and key encoding follow Ghostty's code, MIT, in `LICENSE-GHOSTTY`. The theme files come from iTerm2-Color-Schemes, MIT, with its license in `assets/themes`. JetBrains Mono and its Nerd Font build are under the SIL Open Font License, in `assets/fonts`.

MIT 许可，见 `LICENSE`，与上游 ELYTerminal 一致，本二开版本予以保留并承继其版权声明。制表符绘制与按键编码沿用 Ghostty 的代码，MIT 许可，见 `LICENSE-GHOSTTY`。主题文件来自 iTerm2-Color-Schemes，MIT 许可，其许可声明位于 `assets/themes`。JetBrains Mono 及其 Nerd Font 版本采用 SIL Open Font License，见 `assets/fonts`。
