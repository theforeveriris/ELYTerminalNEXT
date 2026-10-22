# Tasks
# 任务

Progress truth. Top to bottom, one at a time. A task is ticked only after its codex review passes; rounds go in `tasks/reviews.md`, three at most.

进度的真相。自上而下，一次一个。任务只有在 codex 评审通过后才会被勾选；评审轮次记在 `tasks/reviews.md` 中，最多三轮。

## 2. Foundation
## 2. 基础

- [x] T01 Repo, crate, window: git, `.gitignore`, `Cargo.toml` with Ely pinned, `main.rs` opens one window through Ely, logging, `scripts/check.sh`, CI workflow, private Gitea repo, first push.

  - 仓库、crate、窗口：git、`.gitignore`、固定 Ely 版本的 `Cargo.toml`、`main.rs` 通过 Ely 打开一个窗口、日志、`scripts/check.sh`、CI 工作流、私有 Gitea 仓库、首次推送。

## 3. Settings model and storage
## 3. 设置模型与存储

- [x] T02 `Settings` with defaults and ranges; load (missing file, missing field, bad file, bad value) and atomic save; tests.

  - 带默认值与取值范围的 `Settings`；加载（文件缺失、字段缺失、文件损坏、值非法）与原子保存；测试。

## 4. Themes
## 4. 主题

- [x] T03 Built-in themes from upstream files; mapping onto Ely's `Palette`; dark-mode choice; tests.

  - 来自上游文件的内置主题；映射到 Ely 的 `Palette`；深色模式选项；测试。

## 5. Terminal engine
## 5. 终端引擎

- [x] T04 PTY: spawn through alacritty's event loop, OSC 7 tap on the reader, child id, events to the app; tests.

  - PTY：通过 alacritty 的事件循环启动、在读取端做 OSC 7 截取、子进程 id、事件送至应用；测试。

- [x] T05 Shell integration for zsh, bash, fish, PowerShell, injected at launch; tests that zsh and bash report their folder.

  - 面向 zsh、bash、fish、PowerShell 的 shell 集成，在启动时注入；测试 zsh 与 bash 能报告所在目录。

- [x] T06 Terminal view: frame from the grid, colors and styles, wide and combining cells, cursor shapes and blink, font, line height, padding, resize.

  - 终端视图：由网格得到帧、颜色与样式、宽字符与组合字符单元、光标形状与闪烁、字体、行高、内边距、尺寸调整。

- [x] T07 Box drawing, block elements and powerline shapes (U+E0B0–E0B7) painted as geometry; tests.

  - 框线绘制、区块元素与 powerline 形状（U+E0B0–E0B7）以几何图形绘制；测试。

- [x] T08 Keys: xterm encoding, cursor mode, Option as Alt, kitty keyboard protocol, IME; tests.

  - 按键：xterm 编码、光标模式、Option 视作 Alt、kitty 键盘协议、IME；测试。

- [x] T09 Mouse: selection, mouse reporting, wheel and scrollback, clipboard, copy on select, OSC 52, links; tests.

  - 鼠标：选区、鼠标报告、滚轮与回滚缓冲、剪贴板、选中即复制、OSC 52、链接；测试。

- [x] T10 Focus reporting, title, silent bell, scrollback limit, find.

  - 焦点报告、标题、无声响铃、回滚缓冲上限、查找。

## 6. Main window and layout
## 6. 主窗口与布局

- [x] T11 `ColumnShell` with sessions, terminal and right columns; toggles kept in settings; window title; a transparent title bar over the window's background, as Ghostty's.

  - 包含会话列、终端列与右列的 `ColumnShell`；开关状态保存在设置中；窗口标题；像 Ghostty 那样，在窗口背景之上使用透明标题栏。

## 7. Sessions panel
## 7. 会话面板

- [x] T12 Sessions: new, close, select, keys, titles and folders, ended programs, bell marks.

  - 会话：新建、关闭、选中、快捷键、标题与目录、已结束的程序、响铃标记。

## 8. Right panel: Files
## 8. 右侧面板：文件

- [x] T13 Git/Files switch; file tree of the session's folder with Git colors; open; reload on folder and prompt.

  - Git/Files 切换开关；带 Git 配色的会话目录文件树；打开；目录变化与出现提示符时重新加载。

## 9. Right panel: Git
## 9. 右侧面板：Git

- [x] T14 Git through the command line: status, branch, stage, unstage, commit, log, diff; tests on a temp repo.

  - 通过命令行调用 Git：状态、分支、暂存、取消暂存、提交、日志、差异；针对临时仓库的测试。

- [x] T15 Git panel: branch, changes, stage and unstage, commit, recent commits, diff view, errors.

  - Git 面板：分支、更改、暂存与取消暂存、提交、最近提交、差异视图、错误。

## 10. Settings window
## 10. 设置窗口

- [x] T16 Settings window: categories, General rows, live apply and save.

  - 设置窗口：分类、General 行、实时应用并保存。

- [x] T17 Appearance: theme grids with previews, font, cursor with preview, opacity, blur, padding.

  - Appearance：带预览的主题网格、字体、带预览的光标、不透明度、模糊、内边距。

- [x] T18 Terminal rows: scrollback, Option as Alt, copy on select.

  - Terminal 行：回滚缓冲、Option 视作 Alt、选中即复制。

## 11. Background blur
## 11. 背景模糊

- [x] T19 macOS and Windows: `Blurred` or `Transparent` or `Opaque` from settings, applied live.

  - macOS 与 Windows：依据设置取 `Blurred`、`Transparent` 或 `Opaque`，实时应用。

- [x] T20 Wayland: `ext_background_effect_v1` on the window's surface.

  - Wayland：在窗口的 surface 上使用 `ext_background_effect_v1`。

- [x] T21 X11: `_KDE_NET_WM_BLUR_BEHIND_REGION` on the window.

  - X11：在窗口上设置 `_KDE_NET_WM_BLUR_BEHIND_REGION`。

## 12. Menus and shortcuts
## 12. 菜单与快捷键

- [x] T22 macOS menu bar; key bindings on all platforms.

  - macOS 菜单栏；所有平台上的快捷键绑定。

## 13. Acceptance
## 13. 验收

- [x] T23 End-to-end checks, README, acceptance table, CI green on three platforms.

  - 端到端检查、README、验收表、三个平台上 CI 全绿。

## 14. Icon and the macOS app
## 14. 图标与 macOS 应用

- [x] T24 Icon: Ely holding a terminal, on macOS's rounded square and alone for Windows and Linux; `scripts/mac-app.sh` builds `ELYTerminal.app` and its disk image.

  - 图标：手持终端的 Ely，macOS 上置于圆角方形之中，Windows 与 Linux 上单独呈现；`scripts/mac-app.sh` 构建 `ELYTerminal.app` 及其磁盘映像。

## 15. Release
## 15. 发布

- [x] T25 Windows zip with the icon in the executable, Linux tarball with the icon and a desktop entry, the app id on Linux windows, their scripts; release v0.1.0 on Gitea with the three builds.

  - Windows zip，图标嵌入可执行文件；Linux tarball，含图标与 desktop entry；Linux 窗口上的 app id；各自的脚本；在 Gitea 上发布 v0.1.0，含三个构建。

## 16. Fixes from the owner's first run
## 16. 首次运行后的修复

- [x] T26 On Windows every title bar control takes its press, Ely's window buttons too: in Ely, window buttons and header items occlude the drag region; the Git and Files switch moves below the title bar, centered; on Windows and Linux a gear in the title bar opens Settings; on Windows Ctrl-V pastes, as in Windows Terminal; the pointer over the terminal is the arrow; the settings window drops General, whose Shell and Start in lead Terminal, which lists every shortcut, and its categories and page scroll apart; the release's macOS disk image signed with Developer ID and notarized on the signing runner; release v0.1.1 with the three builds.

  - 在 Windows 上，标题栏的每个控件都能响应点击，Ely 的窗口按钮也一样：在 Ely 中，窗口按钮与标题项会遮挡拖拽区域；Git 与 Files 切换开关移到标题栏下方并居中；在 Windows 与 Linux 上，标题栏中的齿轮按钮打开设置；在 Windows 上 Ctrl-V 粘贴，与 Windows Terminal 一样；终端上方的指针是箭头形状；设置窗口去掉 General，其 Shell 与 Start in 前置到 Terminal，Terminal 列出每个快捷键，分类与页面各自滚动；发布版的 macOS 磁盘映像以 Developer ID 签名并在签名 runner 上完成公证；发布 v0.1.1，含三个构建。

## 17. Omarchy
## 17. Omarchy

- [x] T27 On Omarchy 4, an Omarchy theme wears the system's current theme and follows each switch, the default there; the v0.1.1 Linux tarball rebuilt with it, in place of the old one on the release.

  - 在 Omarchy 4 上，Omarchy 主题采用系统当前主题并跟随每次切换，是该环境下的默认；重建含它的 v0.1.1 Linux tarball，替换发布页上的旧文件。

- [x] T28 Blur on Hyprland, which the owner found absent on Omarchy: Hyprland blurs behind a window only while its own `decoration:blur:enabled` is on, which Omarchy 4.0.4 turns off; the PRD, README, AGENTS.md, the acceptance table and the release notes say so, and how to turn it on.

  - Hyprland 上的模糊，owner 发现它在 Omarchy 上并不存在：Hyprland 只有在自身 `decoration:blur:enabled` 开启时才在窗口背后做模糊，而 Omarchy 4.0.4 把它关掉了；PRD、README、AGENTS.md、验收表与发布说明中都写明了这一点以及如何开启。

## 18. herdr
## 18. herdr

- [x] T29 Inside herdr the right column follows herdr's focused pane, which the owner found frozen: herdr keeps its panes' OSC 7, so on macOS and Linux ELYTerminal reads the folder from herdr's socket while herdr's client runs in the session's foreground.

  - 在 herdr 内，右列跟随 herdr 获得焦点的面板，owner 发现它此前是冻结的：herdr 保留了各面板的 OSC 7，因此在 macOS 与 Linux 上，当 herdr 的客户端运行在会话前台时，ELYTerminal 从 herdr 的 socket 读取目录。

- [x] T30 The main window opens at the size it was left at, on macOS, Windows and Linux, which the owner asked for: the size is kept in the settings.

  - 主窗口以离开时的尺寸打开，在 macOS、Windows 与 Linux 上均如此，这是 owner 要求的：尺寸保存在设置中。

- [x] T31 Release v0.1.2 with T29 and T30: version 0.1.2; the macOS disk image, the Windows zip and the Linux tarball, each from its script; the notes say it fixes herdr compatibility and keeps the window's size.

  - 发布 v0.1.2，含 T29 与 T30：版本 0.1.2；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建；发布说明称其修复了 herdr 兼容性并保留窗口尺寸。

- [x] T32 On Windows the right column follows herdr's focused pane too, which the owner found frozen there: ELYTerminal finds herdr's client under the session's shell, reads its arguments and variables, and asks herdr's named pipe.

  - 在 Windows 上右列同样跟随 herdr 获得焦点的面板，owner 发现那里也是冻结的：ELYTerminal 在会话的 shell 之下找到 herdr 的客户端，读取其参数与环境变量，并向 herdr 的命名管道发问。

## 19. Column widths
## 19. 列宽

- [x] T33 The sessions and right columns resize by a drag on their border toward the terminal, on macOS, Windows and Linux, which the owner asked for: in Ely, a `ShellColumn` resizable between a least and a greatest width; the two widths kept in the settings.

  - 会话列与右列通过朝终端方向拖动其边界来调整宽度，在 macOS、Windows 与 Linux 上均如此，这是 owner 要求的：在 Ely 中，`ShellColumn` 可在最小宽度与最大宽度之间调整；两个宽度保存在设置中。

## 20. Prompts, program status and notifications
## 20. 提示符、程序状态与通知

- [x] T34 OSC 133 from the shell integration, which the owner asked for with everything below: zsh, bash, fish and PowerShell mark each prompt's start (A) and end (B), a command's start (C), and its end with the exit status (D); the tap reads any program's OSC 133; each session keeps its last command's status.

  - 来自 shell 集成的 OSC 133，这是 owner 与以下全部内容一并要求的：zsh、bash、fish 与 PowerShell 标记每个提示符的开始（A）与结束（B）、命令的开始（C）以及命令结束及其退出状态（D）；截取器能读取任意程序的 OSC 133；每个会话保存上一条命令的状态。

- [x] T35 Jump to prompt: a prompt's first row is marked once the prompt is drawn; Cmd-Up and Cmd-Down, and Cmd-Shift-Up and Cmd-Shift-Down, on macOS, Ctrl-Shift-Up and Ctrl-Shift-Down elsewhere, scroll to the prompt before or after the view's top, as in Ghostty.

  - 跳转到提示符：提示符绘制完成后即标记其第一行；在 macOS 上为 Cmd-Up 与 Cmd-Down 以及 Cmd-Shift-Up 与 Cmd-Shift-Down，其他平台为 Ctrl-Shift-Up 与 Ctrl-Shift-Down，滚动到视图顶部之前或之后的提示符，与 Ghostty 相同。

- [x] T36 OSC 7501 program status, revision 0.4: reports checked as the spec checks them; records per terminal, by id, with clear and a cap of 256; a new prompt drops working, blocked and idle records, a full reset all; the support query answered with the states and kinds.

  - OSC 7501 程序状态，修订 0.4：按规范的方式校验上报内容；按 id 为每个终端保存记录，支持清除且上限为 256；新提示符丢弃 working、blocked 与 idle 记录，完整重置则丢弃全部；查询以各状态与种类作答。

- [x] T37 OSC 9;4 progress as the root record until an OSC 7501 report; OSC 9, 777 and 99 notifications read, OSC 99's chunks, base64, ids, occasions, actions, close and query included; ConEmu's other OSC 9 commands are no notifications.

  - 在收到 OSC 7501 上报之前，OSC 9;4 进度作为根记录；读取 OSC 9、777 与 99 通知，含 OSC 99 的分块、base64、id、场合、动作、关闭与查询；ConEmu 的其他 OSC 9 命令不算通知。

- [x] T38 Inside herdr, herdr's agent states from its snapshot are the session's records.

  - 在 herdr 内，herdr 快照中的 agent 状态即会话的记录。

- [x] T39 Session rows show the status: working with its progress, waiting by kind, done, error, a failed last command; hover shows each record's program, title and message; a hidden session that starts waiting marks its row.

  - 会话行显示状态：working 及其进度、waiting 按种类、done、error、上一条命令失败；悬停显示每条记录的程序、标题与消息；隐藏的会话开始等待时标记其所在行。

- [x] T40 System notifications on macOS, Windows and Linux through gpui's: a session that starts waiting, finishes or fails, and OSC 9, 777 and 99 requests, while the window is inactive or the session hidden, rate-limited; a press shows the window and the session.

  - 在 macOS、Windows 与 Linux 上通过 gpui 发送系统通知：会话开始等待、完成或失败，以及 OSC 9、777 与 99 请求，在窗口非活动或会话隐藏时发送，并有频率限制；按下通知会显示窗口与会话。

## 21. Release
## 21. 发布

- [x] T41 Release v0.1.3 with T32 to T40: version 0.1.3; the macOS disk image, the Windows zip and the Linux tarball, each from its script; the notes list what changed.

  - 发布 v0.1.3，含 T32 至 T40：版本 0.1.3；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建；发布说明列出变更内容。

## 22. Status marks that move
## 22. 会动的状态标记

- [x] T42 The status marks move, which the owner asked for after aicss.dev's orbs: 3 by 3 dots in the state's color; work ripples, or fills by its progress, waiting dips, a settled state draws its shape, then holds; the window draws again only at the marks' steps while one moves.

  - 状态标记会动起来，这是 owner 在看到 aicss.dev 的 orbs 之后要求的：3 乘 3 的点阵，使用该状态的配色；工作中泛起涟漪，或按进度填充，waiting 时下沉，稳定状态先画出形状再保持不动；只有在标记移动期间，窗口才在标记的各个节拍上重绘。

## 23. Release
## 23. 发布

- [x] T43 v0.1.3 rebuilt with T42, which the owner asked for in place of a v0.1.4: the version stays 0.1.3; the macOS disk image, the Windows zip and the Linux tarball, each from its script, replace the old ones on the release; the tag moves to their commit; the notes say what changed.

  - 重建含 T42 的 v0.1.3，这是 owner 要求的，用来取代 v0.1.4：版本仍为 0.1.3；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建，替换发布页上的旧文件；标签移到它们的提交；发布说明写明变更内容。

## 24. Keys, links and Ely
## 24. 按键、链接与 Ely

- [x] T44 herdr's shortcuts work, which a user found dead: `cmd+shift+ctrl+[` and `]` on macOS, and the same keys as herdr reads them on Linux and Windows: under kitty's flags a key Shift makes a symbol of goes as its own key with Shift, the symbol its alternate, on macOS and Windows; on Windows keys go as win32-input-mode records, which ConPTY asks for and herdr reads; on Linux gpui keeps no key code, which the docs say.

  - herdr 的快捷键可用了，有用户发现它们此前无效：在 macOS 上为 `cmd+shift+ctrl+[` 与 `]`，在 Linux 与 Windows 上则是 herdr 所读取的同一组按键：在 kitty 的标志位下，被 Shift 变为符号的按键以该符号加 Shift 作为独立按键发送，该符号则是它的备用键，在 macOS 与 Windows 上如此；在 Windows 上按键以 win32-input-mode 记录发送，ConPTY 会请求它，herdr 会读取它；在 Linux 上 gpui 不保留键码，文档中说明了这一点。

- [x] T45 Cmd-press on macOS, Ctrl-press on Windows and Linux opens the link under the pointer, which the owner asked for, wherever the terminal shows one: inside herdr, which wraps long URLs at its pane's edge and passes no OSC 8 target on, while herdr takes the mouse the press goes to herdr, which opens its pane's link whole, web links only, as herdr does, Cmd heard as Ctrl on macOS.

  - 在 macOS 上按 Cmd、在 Windows 与 Linux 上按 Ctrl，打开指针下的链接，这是 owner 要求的，终端显示链接的任何位置都适用：在 herdr 内，长 URL 会在面板边缘换行且不会传递 OSC 8 的目标，而 herdr 接管了鼠标，按下会送达 herdr，herdr 打开其面板的完整链接，且仅限网页链接，与 herdr 的行为一致，在 macOS 上 Cmd 被当作 Ctrl。

- [x] T46 Ely peeks over the main window's top edge from behind it, which the owner asked for after the 0.1.3 film, with several faces and moves; a switch in Settings, Terminal, turns her off: a window of her own kept on the edge, macOS's child window, Windows' owned popup, X11's popup moved and stacked, Wayland's anchored popup; ten faces from the icon's paths, following the sessions' marks.

  - Ely 从主窗口背后探出上沿，这是 owner 在 0.1.3 的影片之后要求的，有多种表情与动作；Settings、Terminal 中的开关可以关掉她：她有自己保持在边缘上的窗口，macOS 上为主窗口的子窗口，Windows 上为所属的弹出窗口，X11 上为移动并层叠的弹出窗口，Wayland 上为锚定的弹出窗口；来自图标路径的十种表情，跟随会话的标记。

## 25. Release
## 25. 发布

- [x] T47 Release v0.1.4 with T44 to T46: version 0.1.4; the macOS disk image, the Windows zip and the Linux tarball, each from its script; the notes list what changed.

  - 发布 v0.1.4，含 T44 至 T46：版本 0.1.4；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建；发布说明列出变更内容。

## 26. Ely on Windows
## 26. Windows 上的 Ely

- [x] T48 Ely on Windows stands on the main window's top edge in a box her own size, which a user found broken in 0.1.4: gpui opened her window as large as the display when her box's middle lay off it, so she sat at its bottom and the rest of it took the clicks meant for the windows behind; her window is a bare popup, without the caption, border, corners and shadow gpui's popup carries, and the glue sets her size from her own DPI as well as her place on the pane's top edge.

  - Windows 上的 Ely 以一个属于她自己的尺寸的方框站在主窗口的上沿，有用户发现 0.1.4 中这是坏的：当她的方框中心落在所有显示器之外时，gpui 会把她的窗口开成与显示器一样大，于是她坐在其底部，其余部分则吞掉了本该传给后方窗口的点击；她的窗口是一个纯弹出窗口，不带 gpui 弹出窗口自带的标题栏、边框、圆角与阴影，而粘合代码依据她自己的 DPI 以及她在面板上沿的位置来设置其尺寸。

- [x] T49 Ely moves all the time, as she does in the films, which the owner asked for after 0.1.4, where she stood still and only a press made her wink: rigged from the icon's paths as the films rig her, she bobs, her side locks sway and her ahoge breathes, she blinks and glances on her own, and each mood moves her through that rig, smoothly.

  - Ely 一直在动，就像影片中那样，这是 owner 在 0.1.4 之后要求的，此前她站着不动，只有被按下时才会眨眼：像影片那样从图标路径搭建骨架，她上下浮动，两侧发丝摆动，呆毛呼吸，她自行眨眼与侧目，每种情绪都顺畅地带动这套骨架。

## 27. Release
## 27. 发布

- [x] T50 v0.1.4 rebuilt with T48 and T49, which the owner asked for in place of a v0.1.5: the version stays 0.1.4; the macOS disk image, the Windows zip and the Linux tarball, each from its script, replace the old ones on the release; the tag moves to their commit; the notes give the new SHA-256 and what changed.

  - 重建含 T48 与 T49 的 v0.1.4，这是 owner 要求的，用来取代 v0.1.5：版本仍为 0.1.4；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建，替换发布页上的旧文件；标签移到它们的提交；发布说明给出新的 SHA-256 与变更内容。

## 28. Ely's place, sleep and frame
## 28. Ely 的位置、瞌睡与边框

- [x] T51 Ely stands inside the main window at its bottom left while no room lies above it, which the owner asked for after 0.1.4, where on a window at the screen's top or full screen she stuck between the menu bar and the terminal, on macOS, Windows and Linux: her bottom stands on the pane's top edge while her box fits under the top of the work area of the main window's display, else on the pane's bottom edge, as far in from the left; on Wayland, which tells a client nothing of its place, on the bottom edge while the window is full screen, maximized or tiled at its top as gpui reports it, which it does for a window it draws itself.

  - 上方没有空间时，Ely 站在主窗口内部的左下角，这是 owner 在 0.1.4 之后要求的，此前在屏幕顶部或全屏的窗口上，她会卡在菜单栏与终端之间，在 macOS、Windows 与 Linux 上均如此：她的底边站在面板的上沿上，同时她的方框能放进主窗口所在显示器工作区的顶部，否则就站在面板的下沿上，并尽量远离左侧；在 Wayland 上，客户端无从得知自身位置，因此在窗口全屏、最大化或按 gpui 报告的方式平铺在顶部时站在下沿，而这只适用于 gpui 自己绘制的窗口。

- [x] T52 A press wakes Ely from her doze, which the owner asked for: she dozes after five calm minutes since the later of her mood's start and the last press, so a press wakes her for five more.

  - 一次按下会把 Ely 从瞌睡中唤醒，这是 owner 要求的：自情绪开始与最后一次按下中较晚者起，经过五分钟平静后她便打瞌睡，因此一次按下能再唤醒她五分钟。

- [x] T53 Ely's window shows nothing but her, which the owner asked for after a line showed along its top on macOS: there her window is a borderless panel, without the titled frame gpui's popup carries, whose edge macOS draws; on Windows and Linux her window shows no frame, border or shadow.

  - Ely 的窗口只显示她本人，这是 owner 在 macOS 上顶部出现一条线之后要求的：在 macOS 上她的窗口是一个无边框面板，不带 gpui 弹出窗口自带的带标题边框，其边缘由 macOS 绘制；在 Windows 与 Linux 上她的窗口不显示任何边框、轮廓或阴影。

## 29. Release
## 29. 发布

- [x] T54 Release v0.1.5 with T51 to T53: version 0.1.5; the macOS disk image, the Windows zip and the Linux tarball, each from its script; the notes list what changed.

  - 发布 v0.1.5，含 T51 至 T53：版本 0.1.5；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建；发布说明列出变更内容。

## 30. Ely's drag, updates and the scrollbar
## 30. Ely 的拖动、更新检查与滚动条

- [x] T55 A drag moves Ely along the edge she stands on, which the owner asked for after 0.1.5: she follows the pointer sideways inside the pane, on macOS, Windows and X11 as it moves, on Wayland, whose popup keeps its place, once it lets go; where she stands, in points from the pane's left edge, the settings keep as `ely_left`.

  - 拖动可让 Ely 沿她所站的边缘移动，这是 owner 在 0.1.5 之后要求的：在 macOS、Windows 与 X11 上她随指针在面板内横向移动，在 Wayland 上，由于其弹出窗口位置固定，她在松手后才移动；她所在的位置，以点计、从面板左边缘起算，由设置保存为 `ely_left`。

- [x] T56 Settings › Terminal checks for updates, which the owner asked for after 0.1.5: under Ely, a switch checks ELYTerminal's latest release at start and once a day, on by default, and a button checks at once; the row says what the check found, and a newer release brings a button that opens its page; a newer release an automatic check finds posts a system notification, whose press opens that page.

  - Settings › Terminal 检查更新，这是 owner 在 0.1.5 之后要求的：在 Ely 之下，一个默认开启的开关在启动时以及每天检查一次 ELYTerminal 的最新发布，另有一个按钮可立即检查；该行说明检查结果，若有更新版本则出现一个打开其页面的按钮；自动检查发现更新版本时会发出系统通知，按下通知即打开该页面。

- [x] T58 The terminal shows a scrollbar over its history, which a user asked for in issue #2 after 0.1.5, since a program that takes the mouse takes the wheel too: a thumb along its right edge, in Ely's sizes and colors, stands where the view stands in the history, and a drag on it moves the view through it; none shows without history, as on the alternate screen, and a press on it never reaches the terminal.

  - 终端在历史上显示滚动条，有用户在 0.1.5 之后于 issue #2 中提出，因为接管鼠标的程序也会接管滚轮：一个使用 Ely 尺寸与配色的滑块沿其右缘停在与视图在历史中所处位置相同之处，拖动它即可让视图在历史中移动；没有历史时不显示，如在备用屏幕上；对它的按下绝不会传到终端。

## 31. Release
## 31. 发布

- [x] T57 Release v0.1.6 with T55, T56 and T58: version 0.1.6; the macOS disk image, the Windows zip and the Linux tarball, each from its script; the notes list what changed.

  - 发布 v0.1.6，含 T55、T56 与 T58：版本 0.1.6；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建；发布说明列出变更内容。

## 32. User feedback after 0.1.6
## 32. 0.1.6 之后的用户反馈

- [x] T59 A plain click on the line typed at the shell prompt moves the cursor there, which a user asked for after 0.1.6, as Ghostty's `cursor-click-to-move`: while the shell's OSC 133 marks say it reads a line, a click that does not drag sends Left or Right arrows, one per character, to the clicked cell of the cursor's line, soft wraps joined; never on the alternate screen, under mouse tracking, with a selection, or before the prompt's end.

  - 在 shell 提示符处输入的行上普通点击会把光标移到那里，有用户在 0.1.6 之后提出，与 Ghostty 的 `cursor-click-to-move` 一样：当 shell 的 OSC 133 标记表明它正在读取一行时，不带拖动的点击会发送 Left 或 Right 方向键，每个字符一次，移向光标所在行中被点击的单元，软换行视为相连；在备用屏幕上、鼠标跟踪开启时、有选区时，或在提示符结束之前，绝不如此。

- [x] T66 Files dropped on the terminal type their paths, which a user asked for after 0.1.6, as macOS Terminal does: each path quoted for the shell, joined by spaces, sent as a paste, on macOS, Windows and Linux.

  - 拖放到终端的文件会输入其路径，有用户在 0.1.6 之后提出，与 macOS Terminal 的行为一致：每条路径按 shell 需要加引号，以空格连接，作为一次粘贴发送，在 macOS、Windows 与 Linux 上均如此。

- [x] T60 Fallback fonts, which a user asked for after 0.1.6: `font_fallbacks` in the settings, a list of installed families tried in order for a character Font family lacks, CJK among them; Settings › Appearance lists them under Font family, each an Ely `FontPicker` with a remove button, and an Add button; each saves only when gpui loads it.

  - 回退字体，有用户在 0.1.6 之后提出：设置中的 `font_fallbacks` 是一份已安装字族的列表，当 Font family 缺少某个字符时按顺序尝试，其中包含 CJK 字体；Settings › Appearance 在 Font family 下列出它们，每项是一个带移除按钮的 Ely `FontPicker`，并有一个 Add 按钮；每项只有在 gpui 能加载它时才保存。

- [x] T61 On Windows the IME's candidate window stays at the cursor, which a user found jumping after 0.1.6: find why it moves, fix the cause, and log what the input handler tells the IME. Not reproduced on the owner's PC, so the owner closed it; the caret the IME is told is logged.

  - 在 Windows 上 IME 的候选窗口应停留在光标处，有用户发现 0.1.6 之后它会跳动：找出它移动的原因，修复根源，并记录输入处理器告知 IME 的内容。在 owner 的 PC 上无法复现，因此 owner 关闭了该问题；告知 IME 的插入符位置会被记录。

- [x] T62 Ely's body runs on below the edge she holds, which a user asked for after 0.1.6, where a hop lifted her off the window and left a gap under her collar: her shirt continues down, hidden behind the edge, so a hop shows more of her and never a gap.

  - Ely 的身体在她所抓的边缘下方继续延伸，有用户在 0.1.6 之后提出，此前她一跳跃就离开窗口，在领口下方留下空隙：她的衬衫向下延伸，隐藏在边缘之后，因此跳跃时会露出更多身体，绝不出现空隙。

- [x] T63 A double-click on a session's title renames it, which a user asked for after 0.1.6: the row's title turns into a field, Enter keeps the name, Escape drops it; the name shows in the row, the terminal's header and the window's title over the program's own until renamed again, and an empty name gives the title back to the program; it lives as long as the session.

  - 双击会话标题可重命名，有用户在 0.1.6 之后提出：该行的标题变成输入框，Enter 保留名称，Escape 放弃；名称显示在该行、终端的标题栏以及窗口标题中，覆盖程序自身的标题，直至再次重命名；若名称为空，则把标题交还给程序；名称与会话同存。

- [x] T65 Dragging the window on Windows 10 with Background blur on follows the pointer, which a user found lagging seconds behind in issue #3 after 0.1.5: find where the lag comes from in gpui's acrylic path, fix the cause on Windows 10 alone, log what changes, and reply on the issue.

  - 在 Windows 10 上开启 Background blur 时拖动窗口会跟随指针，有用户在 0.1.5 之后于 issue #3 中发现它会滞后数秒：找出延迟源自 gpui 的 acrylic 路径何处，只在 Windows 10 上修复根源，记录发生的变化，并在 issue 中回复。

## 33. Release
## 33. 发布

- [x] T64 Release v0.1.7 with T59 to T63, T65 and T66: version 0.1.7; the macOS disk image, the Windows zip and the Linux tarball, each from its script; the notes list what changed in English and Chinese.

  - 发布 v0.1.7，含 T59 至 T63、T65 与 T66：版本 0.1.7；macOS 磁盘映像、Windows zip 与 Linux tarball，各自由各自的脚本构建；发布说明以英文与中文列出变更内容。

## 34. Safe defaults and honest answers
## 34. 安全的默认值与诚实的回答

- [x] T67 A title a program writes keeps no control that could disguise it, as future.md's SEC-05 asks: OSC 0 and OSC 2 lose their C0 and C1 controls, their direction controls and their invisible formatting on the way to the window, a newline becomes a space, and a title past 1024 characters is cut.

  - 程序写下的标题不带任何可能伪装它的控制符，这是 future.md 的 SEC-05 所要求的：OSC 0 与 OSC 2 在写进窗口前去掉 C0 与 C1 控制符、方向控制符与不可见格式符，换行变成空格，超过 1024 字符的标题被截断。

- [x] T68 A link opens only what the user may open, as SEC-04 asks: an OSC 8 target and a URL found in a line open under http, https, mailto or file alone; `javascript:`, `data:` and a scheme the terminal knows nothing of stay dead.

  - 链接只打开用户可以打开的东西，这是 SEC-04 所要求的：OSC 8 的目标与行内识别出的 URL 只在 http、https、mailto 与 file 下打开；`javascript:`、`data:` 以及终端不认识的 scheme 一律不打开。

- [x] T69 The terminal says what it is, and no more than it is, as PROTO-01, PROTO-02 and PROTO-03 ask: DA1 answers 62;22 and leaves sixel's 4 and ReGIS's 3 out, so no program tries to send graphics the terminal cannot draw; XTVERSION, which the parser hands to no handler, is read by the tap and answered ELYTerminal with its version; DECRQM answers from the grid's modes and the mouse's, 4 for a mode the terminal knows but never sets and 0 for one it knows nothing of.

  - 终端说自己是什么，不多说一分，这是 PROTO-01、PROTO-02 与 PROTO-03 所要求的：DA1 回答 62;22，不报 sixel 的 4 与 ReGIS 的 3，于是没有程序会去发送终端画不出的图形；XTVERSION 由 tap 读取（解析器不派发它）并回答 ELYTerminal 及其版本；DECRQM 按网格模式与鼠标模式回答，已知但从不设置的答 4，不认识的答 0。

- [x] T70 A session that will not start says why, and what to copy, as SHELL-13 asks: a failed start keeps the program and its arguments, the folder, the error with its causes, and the first eight lines the program wrote itself, and shows them in the window with a Copy button; the words go to the log too.

  - 无法启动的会话会说明原因，并给出可复制的内容，这是 SHELL-13 所要求的：启动失败时保留程序与参数、目录、错误及其成因，以及程序自己写下的前八行，在窗口中显示并带一个复制按钮；这些文字也记入日志。

## 35. Paths, settings and a tab of its own
## 35. 路径、设置与一个属于自己的标签

- [x] T71 A path is copied in the form the user names, as PATH-08 and PANEL-09 ask: the Files panel's copy button opens a menu with the path as the system writes it, as Git Bash reads it, as WSL reads it, quoted for a shell, or as a file URI; what a shell reads specially is escaped rather than dropped.

  - 路径以用户指定的形式被复制，这是 PATH-08 与 PANEL-09 所要求的：文件面板的复制按钮展开一份菜单，路径可按系统的写法、Git Bash 的写法、WSL 的写法、加引号后给 shell、或作为 file URI 复制；shell 会特别解读的字符被转义而非丢弃。

- [x] T72 A settings file may name a variable the environment holds, as CFG-06 asks: `${VAR}` expands once in the shell, editor, start-in and last-folder settings, read from the environment and never from a command; a variable with no value fails the load and names itself.

  - 设置文件可以写出环境变量的名字，这是 CFG-06 所要求的：`${VAR}` 在 shell、editor、start_in_folder 与 last_folder 中展开一次，只从环境变量读取，绝不执行命令；没有取值的变量使读取失败并点名自己。

- [x] T73 A session opens again in a tab of its own, in the folder it was last in, as TAB-13 asks: Ctrl-Shift-K, or Cmd-Shift-K on macOS, clones the shown session with the same program and variables in the folder it last reported, and no scrollback or running state comes with it.

  - 会话可以在一个属于自己的标签里再次打开，并落在它最后所在的目录，这是 TAB-13 所要求的：Ctrl-Shift-K，macOS 上为 Cmd-Shift-K，以同样的程序与环境变量在会话最后报告的目录里克隆它，回滚缓冲与运行状态都不跟随。

- [x] T74 A first run writes a config file that says what each field sets, as CFG-16 asks: with neither `config.toml` nor `settings.json` there, the app writes a `config.toml` of the defaults, each field under a comment naming what it sets and what else it could be; an existing file is never overwritten.

  - 首次运行写出一份能说明每个字段设置什么的配置文件，这是 CFG-16 所要求的：在既无 `config.toml` 也无 `settings.json` 时，应用写出一份填入默认值的 `config.toml`，每个字段上方一行注释写明它设置什么以及还能取什么值；已存在的文件绝不被覆盖。

## 36. Commands, marks and where a session sits
## 36. 命令、标记与会话所在之处

- [x] T75 A command is kept with the line that was typed, and can be walked, listed and exported, as BLOCK-03, BLOCK-05, BLOCK-10, BLOCK-11 and STAT-19 ask: the line typed between the shell's prompt and its command is read from the grid across soft wraps; Ctrl-Alt-Up and Down walk the commands that ended, Ctrl-Shift-Alt-Up and Down only those that failed, and Ctrl-Shift-Alt-E copies them as Markdown; a summary counts what ran and what failed and names the slowest, and the export comes as text, Markdown or JSON.

  - 命令与输入的那一行一起保存，并可以逐条走过、列出与导出，这是 BLOCK-03、BLOCK-05、BLOCK-10、BLOCK-11 与 STAT-19 所要求的：提示符与命令之间输入的那一行跨软换行从网格读出；Ctrl-Alt-上下 走过已结束的命令，Ctrl-Shift-Alt-上下 只走失败的，Ctrl-Shift-Alt-E 以 Markdown 复制它们；摘要统计跑了多少、失败多少并点名最慢的一条，导出有文本、Markdown 与 JSON 三种。

- [x] T76 The marks a shell writes read the same under OSC 633, and the old alternate screens read as the one vte holds, as PROTO-13 and PROTO-10 ask: VS Code's OSC 633 marks read like 133, with aid, cl, cmdline_url, k and err read past rather than refusing the mark, aid and err kept for the session; modes 47, 1047 and 1048 are passed on to the parser as the 1049 they stand for.

  - shell 写的标记在 OSC 633 下同样读取，旧备用屏按 vte 认识的模式读取，这是 PROTO-13 与 PROTO-10 所要求的：VS Code 的 OSC 633 标记与 133 一样读取，aid、cl、cmdline_url、k 与 err 被读过去而不是让标记作废，aid 与 err 留给会话；模式 47、1047 与 1048 以它们所代表的 1049 转给解析器。

- [x] T77 A session says where it sits, as STAT-14, STAT-17 and CONN-18 ask: the terminal's column shows the branch the shown session's folder sits in, read from the nearest `.git/HEAD` or a worktree's gitfile; a line under the terminal says the connection, the grid's cells and the branch, each left out where it says nothing; a session row on another machine leads with that machine — `WSL: archlinux`, `ssh: host`.

  - 会话说明自己在哪里，这是 STAT-14、STAT-17 与 CONN-18 所要求的：终端列显示当前会话所在目录的 Git 分支，读最近的 `.git/HEAD` 或 worktree 的 gitfile；终端下方一行显示连接、网格的列数与行数以及分支，没有内容的部分就不出现；在另一台机器上的会话行以那台机器开头——`WSL: archlinux`、`ssh: host`。

## 37. Settings that build on each other
## 37. 互相叠放的设置

- [x] T78 A settings file may build on the files it names, as CFG-05 asks: `config.toml` names others with `include`, one or a list, each relative to itself; they are laid over one another first and the file's own fields go over theirs, so shared settings live in one place and a project's file says only what it changes; a loop stops at a file already read, and a missing one names the file that asked for it.

  - 设置文件可以建立在它点名的文件之上，这是 CFG-05 所要求的：`config.toml` 用 `include`（一个名字或一列名字，相对自身路径）点名其他文件；它们先互相叠放，本文件自己的字段再压在上面，所以公共设置只写一处、项目文件只写改动；成环时遇到已读过的文件就停，缺失的文件报错时点名是哪个文件要它。

## 38. What this machine came to
## 38. 这台机器的状况

- [x] T79 The terminal can say what it found of this machine, on demand: Ctrl-Shift-Alt-H reads the settings file as a fresh start would, counts the four shell integration scripts, and asks each shell the scan found whether it still stands where it stood; the report shows over the terminal, one finding a line, a dash for what stands and a bang for a fault, and goes to the log as well.

  - 终端可以按需说明它在这台机器上找到了什么：Ctrl-Shift-Alt-H 像一次全新启动那样读取设置文件、数一数四份 shell 集成脚本、逐个询问扫描到的 shell 是否仍在原处；报告叠在终端上，一行一条，`-` 表示正常、`!` 表示有问题，同时也写入日志。

## 39. Panes and tabs that stay where they are put
## 39. 停在原处的窗格与标签

- [x] T80 A pane keeps the share it was dragged to, as TAB-15 asks: a pane's width and its row's height are weights that the layout grows by, the border after a pane drags its row's widths and the border under a row drags the rows' heights, a double press on either evens them again, and Ctrl-Shift-Alt-E does the same from the keyboard; a drag stops at five percent so a pane can never vanish.

  - 窗格保留被拖到的份额，这是 TAB-15 所要求的：窗格的宽度与所在行的高度都是权重，布局按权重伸缩；窗格后的边界可拖动该行的宽度，行下的边界可拖动各行的高度，双击任一边界重新均分，Ctrl-Shift-Alt-E 是键盘上的同一件事；拖动到百分之五为止，窗格不会消失。

- [x] T81 A tab moves among its neighbours, and a pinned tab stays in front, as TAB-03 and TAB-04 ask: Ctrl-Shift-Alt-Up and Down move the shown tab a place, Ctrl-Shift-Alt-P pins it, which puts it before the rest and stops it moving or being crossed; the row shows a star beside a pinned tab.

  - 标签可在相邻之间移动，固定的标签停在前面，这是 TAB-03 与 TAB-04 所要求的：Ctrl-Shift-Alt-上下 把当前标签移动一位，Ctrl-Shift-Alt-P 固定它，于是它排在前面、不再移动也不被越过；固定的标签行上有一颗星。

## 40. What a program may write, and what failed
## 40. 程序能写什么，以及哪些失败了

- [x] T82 A program cannot fill the clipboard, and a session names the commands that failed, as SEL-08, STAT-19, FIND-08 and INP-14 ask: an OSC 52 write over 100 KiB is refused and logged, since a program writes without asking; a session's hover names the commands that failed in it, one by its own line and several by count; the settings' shortcut list carries every key this fork adds, and Ctrl-Shift-/ opens it.

  - 程序不能把剪贴板填满，而会话会点名失败过的命令，这是 SEL-08、STAT-19、FIND-08 与 INP-14 所要求的：OSC 52 写入超过 100 KiB 时整条拒绝并记日志，因为程序写入时并不询问；会话的悬停点名其中失败过的命令，一条时直接写出，多条时给出条数；设置的快捷键表收录了本 fork 新增的每个按键，Ctrl-Shift-/ 可打开它。

## 41. The window's title
## 41. 窗口标题

- [x] T83 The window's title is a template the user writes, as UI-17 asks: `{title}`, `{cwd}` and `{shell}` stand for the session's name, its folder as the sessions column writes it and its program's name, and a template that names none of them falls back to the session's name; Settings › Terminal carries the field, and the seeded `config.toml` explains the variables.

  - 窗口标题是用户自己写的模板，这是 UI-17 所要求的：`{title}`、`{cwd}`、`{shell}` 分别代表会话名、按会话栏写法显示的目录、以及程序名，一个变量都不含时退回会话名；设置 › Terminal 有对应输入框，种子 `config.toml` 里说明了这几个变量。
## 42. The colors of the user's own
## 42. 用户自己的颜色

- [x] T84 The custom theme is a picker for every role it colors, as UI-02 asks: the settings page groups them as the window, the cursor, the selection and the sixteen ANSI colors, each cell a picker that writes as it moves and shows its hex, and a button that starts the colors from the theme in use rather than from a fixed default; a name and Save keep them as a theme of their own, listed in both theme grids, a repeated name taking the next number, and one can be deleted, putting the custom theme back on if it was the one worn.

  - 自定义主题为它上色的每个角色都有一只取色器，这是 UI-02 所要求的：设置页把它们分成界面、光标、选区与十六个 ANSI 颜色，每格一只取色器，移动即写入并标出十六进制；另有一个按钮从正在使用的主题载入颜色，而不是从一个固定的默认值开始；起名保存就得到一份独立主题，在两个主题网格里都列出，重名自动加序号，也可以删除，删的是正在用的那份时退回自定义主题。
## 43. A shell of the user's choosing
## 43. 自己挑一个 shell

- [x] T85 The shell a session starts is picked, not typed from memory, as SHELL-02 and SHELL-04 ask: the Terminal settings' Shell is a list of every shell the scan found, with its version, chosen writes its path, and the field for a path stays under it for one the scan did not find; the sessions column's plus, beside its filter, opens a menu of the session the settings would start, then every scanned shell and every ssh host, each opening a tab of its own in the folder a new session would start in.

  - 会话用哪个 shell 是选出来的，不必凭记忆写，这是 SHELL-02 与 SHELL-04 所要求的：终端设置里的 Shell 是扫描到的每个 shell 的列表并带版本，选中即写入其路径，扫描不到的 shell 仍可在下面的路径框里填写；会话栏筛选框旁的「+」展开一份菜单，先是默认会话，接着每个扫描到的 shell 与每个 ssh 主机，各自开一个标签，起始目录与新建会话一致。
## 44. A tab that folds
## 44. 会折叠的标签

- [x] T86 A tab folds its panes away when its own row is pressed, as TAB-04 and TAB-30 ask: a second press unfolds them, pressing another tab shows that one open, and a folded row keeps its pane count beside its name; the fold is kept in the session file and a file that never said stays open, while the keyboard still walks the panes a folded tab holds, since folding is what the column shows, not what the window holds.

  - 按一下标签自己的那一行，它下面的窗格就折起来，这是 TAB-04 与 TAB-30 所要求的：再按一下展开，按别的标签则切过去并展开它，折叠的行在名字旁始终显示窗格数；折叠写进会话文件，没写过的文件保持展开；键盘仍会走进折叠标签里的窗格，因为折叠只是侧栏画出来的样子，不是窗口里没有的东西。
