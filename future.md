# ELYTerminalNEXT 功能蓝图（future.md）

> 版本 2 · 2026-10-21
> v1 回答的是「现代终端该有什么」；v2 回答的是「具体做成什么样、先做哪个、怎么验收」。
> 本文是功能库 + 规格草案，不是已实现契约。已承诺的行为以 `PRD.md` 为准，执行拆分进 `TASKS.md`。

---

## 0. 阅读指南

### 0.1 这份文档怎么用

- **按编号引用**：`SHELL-05`、`TAB-16` 这类编号可以直接写进 issue、提交信息和 `TASKS.md`。
- **状态**：`已有` / `部分` / `缺`，基于 2026-10-21 的 `src/` 核对。
- **优先级**：P0（两周内）/ P1（一到两个月）/ P2（一个季度）/ P3（实验或以后）。
- **成本**：S（1–3 天）/ M（约 1 周）/ L（2–4 周）/ XL（一个月以上）。
- **验收**：每条重点功能给出可观察的结果，不写「体验更好」这种无法验收的话。

### 0.2 五条产品原则

1. **终端不抢 shell、tmux、编辑器的活。** 终端负责显示、输入、连接、状态聚合；补全、历史、多路复用交给已有工具。
2. **本地优先，零遥测。** 配置、凭据、日志都在本机；任何联网功能默认关闭或显式触发。
3. **协议诚实。** 没实现 sixel 就不在 DA1 里报 sixel；不知道的 DECRQM 就答「未知」，不要假装。
4. **每个功能都能关。** 宠物、动画、通知、AI、实验特性都要有开关；默认值偏安静。
5. **先解决每天碰到的。** 一个人用的终端，排序标准是「今天会不会用到」，不是「竞品有没有」。

### 0.3 版本主题（建议）

| 版本 | 主题 | 主要交付 |
|---|---|---|
| v0.2 | 连接 | shell 目录、profile、WSL/Git Bash、路径映射、SSH 主机列表、安全默认值 |
| v0.3 | 工作台 | 会话恢复、命令面板、键位自定义、配置文件热重载、标签/分屏人体工学 |
| v0.4 | 正确性 | DA1/DECRQM/XTVERSION、字素簇、modifyOtherKeys、鼠标 1015/1016、2027/2048 |
| v0.5 | 远程 | SFTP、端口转发、远程编辑、串口、容器 profile、断线重连 |
| v0.6 | Agent | 命令块、agent 看板、审批提醒、通知聚合、失败诊断 |
| v0.7 | 打磨 | 主题导入、快速终端、多窗口、无障碍、i18n |
| v1.0 | 稳定 | 性能预算、协议测试、跨平台矩阵、文档与打包 |

---

## 1. 现状基线

### 1.1 已有能力（已核对源码）

| 能力 | 现状 | 位置 |
|---|---|---|
| VT 引擎 | `alacritty_terminal` 0.26 + 自有 pty（Unix `rustix-openpty`，Windows ConPTY） | `src/term/pty.rs`、`src/term/unix.rs` |
| 标签 | 有，形态是**左栏会话树里的标签行**，不是顶部标签条；无拖拽排序、无溢出处理 | `src/workspace/layout.rs`、`view.rs` |
| 分屏 | 有，横/纵；比例固定为一半，不能拖分隔条，无窗格放大 | `layout.rs` 的 `Pane { width, height }` |
| Shell 集成 | zsh / bash / fish / pwsh；OSC 7 + OSC 133 | `src/term/shell.rs`、`assets/shell-integration/` |
| 程序状态 | OSC 7501 rev 0.4、OSC 9;4、herdr 状态 | `src/term/status.rs`、`herdr.rs` |
| 通知 | OSC 9 / 777 / 99、系统通知、限流 | `src/term/notify.rs` |
| 超链接 | OSC 8 + URL 检测 + 规则打开 | `src/term/links.rs` |
| 剪贴板 | 复制/粘贴、括号粘贴、复制即选、OSC 52 写入（读取默认关闭） | `input.rs`、`pointer.rs`、`view.rs:317` |
| 键盘 | legacy + kitty 键盘协议 + win32-input-mode + IME | `keys.rs`、`win32.rs` |
| 鼠标 | 9/1000/1002/1003 + SGR/UTF-8 + 滚轮 + 点击移动 | `mouse.rs`、`pointer.rs` |
| 搜索 | 正则、跨滚动缓冲、组合标记 | `find.rs` |
| 文件面板 | 跟随 cwd、文件树、Git 标记 | `files.rs`、`workspace/events.rs:34` |
| Git 面板 | status、分支、diff、stage/commit | `git/panel.rs`、`git/diff.rs` |
| 重命名 | 会话级就地重命名 | `workspace/rename.rs` |
| 设置 | 只有 `settings.json`，无外部编辑、无热重载、无 profile | `src/settings/store.rs` |
| 快捷键 | 全部硬编码在 `bind_keys` | `src/workspace.rs:74` |
| 默认 shell（Windows） | 固定 `powershell.exe`；设置里只有**一个路径字符串** | `src/term/shell.rs:154` |
| 会话恢复 / 多窗口 / 命令面板 / 快速终端 / 关闭确认 | 无 | — |

### 1.2 本机实测环境（2026-10-21）

| 项 | 实测 | 对实现的含义 |
|---|---|---|
| 系统 bash | `C:\Windows\System32\bash.exe`（**WSL 启动器，不是 Git Bash**） | 按文件名识别会误判 |
| Git Bash | `C:\Program Files\Git\bin\bash.exe`、`...\usr\bin\bash.exe` | 要按安装根 + 路径形态区分 |
| WSL | WSL2，默认 `archlinux`，另有 `Ubuntu-22.04`，均 Stopped | 注册表可枚举；首次启动慢 |
| PowerShell | 只有 Windows PowerShell 5.1，无 `pwsh.exe` | 不能假设 `pwsh` 存在；无 `-WorkingDirectory` |
| cmd | `%COMSPEC%` = `C:\Windows\System32\cmd.exe` | 用 `%COMSPEC%`，不要 `where cmd` |
| 其他 shell | 未安装 nu / fish / zsh / elvish / xonsh | 列表要允许为空 |

### 1.3 需要纠正的旧说法

1. `FEATURES.md` / `PLAN.md` 说「alacritty 不支持 reflow」——**0.26 已支持**。`Term::resize` 调用 `grid.resize(!is_alt, …)`，主屏按 `WRAPLINE` 重排，备用屏不重排（正确）。剩下的是宽字符、tab、OSC 8 id、选区在重排后的重映射测试。
2. `FEATURES.md` 说「没有标签、没有分屏」——已经有，只是标签在左栏树里。是否加顶部标签条见 `TAB-02`。
3. `PLAN.md` 设想「顶部标签条」，当前实现是树形标签。两者不必都做，见 `TAB-01` / `TAB-02` 的取舍。

### 1.4 本轮已做（2026-10-22）

> 以下按 `future.md` 的编号记录这一轮落地的部分；没列的保持原状态。

- **SHELL-01 / 04 / 05**：启动时扫描已知安装路径、`PATH`、Git for Windows 注册表项与 `wsl.exe -l -v`；Git Bash 与 System32 的 WSL 启动器分开识别；WSL 发行版按本地化表头安全的列解析。见 `src/term/shell/scan.rs`。
- **SHELL-03 / 07**：设置未指定 shell 时选评分最高的本地 shell；WSL 用 `-d`、`--cd` 启动，PowerShell 5.1/7 共用 `-NoLogo -NoExit -ExecutionPolicy Bypass -Command`。见 `src/term/shell.rs`。
- **SHELL-12**：起始目录支持 `Home` / `当前会话` / `固定目录` / `上次目录`，目录不存在时回退到 home；设置窗口可选择并填写固定目录。见 `src/settings/mod.rs`。
- **PATH-01 / 03 / 04 / 05**：新增 `Location` 模型与 Windows ↔ WSL ↔ Git Bash 路径映射；起始目录校验与回退也走它。见 `src/location.rs`。
- **PATH-02**：除 OSC 7 外，读取 ConEmu `OSC 9;9` 与 iTerm2 `OSC 1337;CurrentDir`。
- **PATH-07**：拖放路径按目标 shell 转换：WSL 得 `/mnt/c/...`，Git Bash 得 `/c/...`，再按 shell 语法引用。
- **FIND-03**：单行选区会预填到搜索框。
- **TAB-16 / 25**：`Ctrl/Cmd+Shift+Enter` 放大当前窗格，`Ctrl/Cmd+Shift+Z` 隐藏两栏进入 zen。
- **BLOCK-01 / 07**：OSC 133 标记组成命令块，记录时长与退出码；会话行显示最近一条命令的耗时与失败码。见 `src/term/blocks.rs`。
- **STAT-10**：超过 10 秒的命令结束时，在会话不可见时发系统通知。
- **CONN-01 / 02 / 03**：读取 `~/.ssh/config`（含 `Include`，跳过通配 Host），`ssh -G` 解析生效值并按配置 mtime 失效缓存；左栏列出主机，点一行开新标签执行 `ssh <host>`。
- **CFG-02**：设置文件每秒检查 mtime，外部编辑保存后 700ms 内生效；坏文件保留旧设置且只报一次。
- **SHELL-14**：设置窗口可重新写入 shell 集成脚本；`repair` 会先清空再写，安装目录可查看。
- **INP-10**：鼠标报告支持 1005、1006、1015、1016（1016 用像素坐标），并支持横向滚轮 66/67。
- **UI-02**：设置里的「Import…」读取 iTerm2 `.itermcolors`、Windows Terminal JSON 或 Ghostty 主题，落到自定义配色并切换过去；「Export…」把当前主题写成 Ghostty 文件。
- **PANEL-09 / 10**：文件面板选中一项后可复制其路径，或在资源管理器/Finder/xdg-open 中显示。
- **SHELL-02**：启动后在后台向每个 shell 询问版本（PowerShell 5.1/7、bash、cmd、WSL、nu 等），设置里的 shell 列表显示成 `Name (version)`。
- **TAB-26**：主窗口的大小与位置都写进设置；恢复时限制在主屏可见区域内，屏幕变化不会把窗口留在外面。
- **会话恢复（v0.3 草案的第一步）**：标签名与每个标签的当前 cwd 写入 `session.json`（原子替换，最多 16 个标签）；启动时按设置恢复标签并回到上次的标签；Terminal 设置里可关闭恢复。
- **TAB-08**：`Ctrl+Tab` / `Ctrl+Shift+Tab` 按最近使用切换标签；顺序切换移到 `Ctrl+PageDown` / `Ctrl+PageUp`。
- **FIND-06**：滚动条上以细线标出搜索命中的历史位置，当前结果仍在主屏高亮。
- **A11Y-06（部分）**：新增「界面字号」（10–24px），只缩放侧栏、设置与标签等 UI，终端的字体大小仍由 Font size 单独控制。
- **PANEL-06 / PATH-09**：文件面板双击文件时，若设置了 Editor 命令（如 `code -g {file}:1`）就用它打开，否则交给系统默认程序；命令支持引号、反斜杠和 `{file}` 占位符。
- **CFG-14**：`settings.json` 解析失败时给出文件名、行号、列号与字段路径，旧设置保持不变。
- **TAB-17（交换部分）**：`Ctrl+Alt+方向`（macOS `Cmd+Alt+Shift+方向`）把当前窗格与相邻窗格交换，焦点跟着窗格走。
- **CFG-04（部分）**：未知设置字段不再让整个文件读取失败；会记录警告，内存里保留，下次保存时丢弃。
- **CFG-01**：配置目录里有 `config.toml` 时优先读取并把设置写回 TOML（`[custom_colors]` 等表格也支持），否则继续用 `settings.json`；热重载按当前文件的扩展名选择解析器。
- **TAB-26**：除了大小与位置，窗口是否最大化也会记住；最大化时只更新该标记，不覆盖还原尺寸。
- **STAT-13（部分）**：响铃时窗格边框闪一下（180ms），可在 Terminal 设置关闭。
- **A11Y-05（部分）**：新增「减少动态效果」；开启后停止会话状态点的动画、光标闪烁与响铃闪烁。
- **STAT-03（部分）**：新增「勿扰模式」；开启后不再发送系统通知，窗口内状态点与响铃闪烁仍保留。
- **TAB-30（部分）**：会话栏顶部可按名称或目录筛选；匹配的标签保留，未匹配的标签与窗格行隐藏。
- **TAB-12（部分）**：关闭仍有前台命令的会话或标签时会先确认（Unix 用前台进程组判断；Windows 暂不拦截）；Terminal 设置里可关闭该确认。
- **SEL-07 / INP-09（部分）**：粘贴含 ESC 等控制字符或超过 4000 字符时，先在窗格左上角显示预览、行数与 Paste/Cancel，确认后才以括号粘贴发送；换行统一转成 CR，避免多行直接执行。
- **SEC-05**：OSC 0 / 2 的标题在写进窗口前去掉 C0 / C1 控制符、方向控制符与不可见格式符，换行变空格，超过 1024 字符的截断；程序无法用转义序列结束标题再自己设定颜色。见 `src/term/titles.rs`。
- **SEC-04**：OSC 8 目标与行内识别出的 URL 只在 http / https / mailto / file 下可点；`javascript:`、`data:` 与不认识的 scheme 一律不打开。见 `src/term/links.rs`。
- **PROTO-01 / 02 / 03**：DA1 按真实能力回答 `62;22`（VT220 + ANSI 颜色），不报 sixel 的 4 与 ReGIS 的 3；XTVERSION 由 tap 读取（vte 不派发它）并回答 `ELYTerminal(版本)`；DECRQM 按网格模式与鼠标模式回答，已知但不设置的答 4，不认识的答 0。见 `src/term/device.rs`。
- **SHELL-13**：启动失败时保留程序与参数、目录、错误及其成因，以及程序自己写下的前 8 行；窗口里显示这份报告并带一个复制按钮，日志里也记一份。见 `src/term/pty.rs` 的 `Failed` 与 `src/workspace.rs`。
- **PATH-08 / PANEL-09**：文件面板的复制按钮展开一份菜单，路径可按系统写法、Git Bash 写法、WSL 写法、加引号或 `file:` URI 复制；shell 会特别解读的字符被转义而不是丢弃。见 `src/location.rs` 的 `written`。
- **CFG-06**：`shell`、`editor`、`start_in_folder` 与 `last_folder` 里的 `${VAR}` 从环境变量展开一次，不执行任何命令；变量没有取值时报错并点名该变量。见 `src/settings/store.rs` 的 `expand`。
- **TAB-13**：`Ctrl/Cmd-Shift-K` 把当前会话克隆到一个新标签，用同样的程序与环境变量、在它最后报告的目录里启动；回滚缓冲与运行状态不跟随。见 `src/workspace/tabs.rs` 的 `clone_of`。
- **CFG-16**：既无 `config.toml` 也无 `settings.json` 时，首次运行写出一份 `config.toml`，每个字段上方一行注释说明它设置什么、可选值有哪些；已存在的文件绝不被覆盖。见 `src/settings/seed.rs` 的 `commented`。
- **BLOCK-03 / 05 / 10 / 11、STAT-19**：命令块保留在提示符与命令之间输入的那一行（跨软换行从网格读出）；`Ctrl-Alt-↑/↓` 走上一条/下一条命令，`Ctrl-Shift-Alt-↑/↓` 只走失败的，`Ctrl-Shift-Alt-E` 以 Markdown 复制全部命令；`summary` 统计跑了多少条、失败多少条并点名最慢的一条；导出支持文本、Markdown 与 JSON。见 `src/term/blocks.rs` 与 `src/term/view/blocks.rs`。
- **PROTO-13**：VS Code shell 集成写的 OSC 633 标记与 133 同样读取；标记携带的 `aid`、`cl`、`cmdline_url`、`k`、`err` 不再让整条标记作废，其中 `aid` 与 `err` 被读出来交给会话。见 `src/term/marks.rs`。
- **PROTO-10**：模式 47、1047、1048 与 1049 要求同一块备用屏，而 vte 不认识前三个，于是 tap 把它们改写成 `CSI ? 1049 h/l` 再送给解析器。见 `src/term/tap/modes.rs`。
- **STAT-17**：终端列标题显示会话所在目录的 Git 分支，读最近的 `.git/HEAD` 或 worktree 的 gitfile；不在工作树里或处于游离 HEAD 时不显示。见 `src/workspace/badge.rs`。
- **STAT-14 / CONN-18**：终端下方一行显示会话所在的连接（`WSL: archlinux`、`ssh: host`）、网格的列数与行数、以及所在分支；会话行在非本机时以连接开头。见 `src/workspace/status.rs` 与 `src/sessions.rs` 的 `connection`。
- **CFG-05**：`config.toml` 可用 `include`（一个名字或一列名字，相对自身路径）分成多份；被点名的文件先互相叠放，本文件自己的字段再压在上面，所以公共设置只写一处、项目文件只写改动；互相引用成环时遇到已读过的文件就停，缺失的文件报错时点名是哪个文件要它。见 `src/settings/include.rs`。
- **健康检查（future.md §8 第 20 条）**：`Ctrl-Shift-Alt-H` 按一次就能看到这台机器的状况：设置文件能否读出（含未知字段）、四份 shell 集成脚本是否齐全、扫描到的每个 shell 是否仍在原处，报告叠在终端上、一行一条，`-` 表示正常、`!` 表示有问题，同时写入日志。见 `src/health.rs` 与 `src/workspace/health.rs`。
- **TAB-15**：窗格的宽度与行的高度改成权重，布局按份额伸缩而不是一律对半分；行内窗格之间的边界可横向拖动改宽度，行与行之间的边界可纵向拖动改高度，双击任一边界把该行与各行重新均分，`Ctrl-Shift-Alt-E` 是键盘上的同一件事；拖动到 5% 为止，窗格不会消失。见 `src/workspace/layout.rs` 与 `src/workspace/panes.rs`。
- **TAB-03 / TAB-04**：`Ctrl-Shift-Alt-↑/↓` 在会话栏里上移/下移当前标签，`Ctrl-Shift-Alt-P` 固定或取消固定；固定标签排在前面、不再移动也不被越过，行上以 `★` 标出。见 `src/workspace/layout.rs` 的 `Tab::move_by` 与 `Tab::pin`。
- **SEL-08 / SEC-03 / PROTO-16**：OSC 52 写入超过 100 KiB 时整条拒绝并记警告；读取仍默认关闭。见 `src/term/clipboard.rs`。
- **STAT-19 / FIND-08**：会话行的悬停列出该会话里失败过的命令——一条时直接点名，多条时给出条数再逐条列出；`Ctrl-Shift-Alt-↑/↓` 在终端里跳到上一条/下一条失败命令。见 `src/sessions.rs` 的 `hover` 与 `src/term/blocks.rs` 的 `step`。
- **INP-14（部分）**：设置 › Terminal 的快捷键表补上本 fork 新增的全部按键并可搜索，`Ctrl-Shift-/` 直接从终端打开它。见 `src/settings/window/terminal.rs`。
- **UI-17**：窗口标题是一个模板，`{title}`、`{cwd}`、`{shell}` 分别代表会话名、按会话栏写法显示的目录、以及程序名（去掉扩展名）；模板为空或一个变量都不含时退回会话名。设置 › Terminal 有对应输入框，种子 `config.toml` 里也有说明。见 `src/settings/title.rs` 与 `src/workspace/title.rs`。
- **UI-02（自定义主题）**：设置里的自定义配色恢复成四组自适应格子——界面、光标、选区、ANSI 颜色，每格一个取色器并写明十六进制；「从当前主题载入」把正在使用的主题颜色填进来，改色即生效。起一个名字按「保存为主题」，这份颜色就成为独立主题，出现在浅色与深色两个主题网格里；重名自动加序号（`名字 2`），可删除，删掉正在用的那份时退回自定义主题。见 `src/settings/window/custom.rs`、`src/themes/custom.rs` 的 `SavedTheme` 与 `src/settings/window/themegrid.rs`。
- **SHELL-02 / SHELL-04（选择 shell）**：设置 › Terminal 的 Shell 是一个下拉，列出扫描到的每个 shell 与它的版本，选中即写入其路径；下面仍保留手填路径的输入框，扫描不到的 shell 照样能用。会话栏筛选框左边的「+」按一下展开一份菜单：第一项是新建默认会话，接着每个扫描到的 shell、每个 ssh 主机各占一行，选一个就开它自己的标签，起始目录与新建会话一致；分屏按钮也从栏顶那条按不动的标题行挪到了这里。见 `src/settings/window/terminal.rs` 与 `src/workspace/open.rs`。
- **TAB-04 / TAB-30（标签折叠）**：按一下标签自己的那一行就把它下面的窗格折起来，再按一下展开，按别的标签则切过去并展开它；折叠的标签行右侧始终显示它有几个窗格，折叠状态写进 `session.json`，旧文件没写就是展开。折叠只是视觉上的：`Ctrl+Shift+←/→` 仍会走到折叠里的窗格，新分出的窗格会把标签展开。见 `src/workspace/layout.rs` 的 `Tab::collapsed` 与 `src/workspace/tabs.rs` 的 `tap_tab`。

---

## 2. 功能库

> 读法：先看「状态 + 优先级」决定要不要做，再看「行为与细节」确认做成什么样，「实现要点」是给实现者的提示。

### 2.1 SHELL：发现并启动本机的每一个 shell

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| SHELL-01 | Shell 目录扫描 | 缺 | P0 / M | 启动后后台列出本机可用的 shell：PowerShell、cmd、Git Bash、WSL 发行版、MSYS2、Cygwin、nu、elvish、xonsh。每条带来源与置信度 | 注册表 + `where.exe` + 已知路径；TTL 5 分钟；`source` 字段解释「为什么列出它」 |
| SHELL-02 | 版本探测 | 缺 | P1 / S | 设置里显示 `PowerShell 5.1`、`pwsh 7.4`、`Git 2.45`、`WSL 2 / Ubuntu 22.04`；失败显示「未知」 | 异步跑 `--version` / 读文件版本；不要在首屏同步执行 |
| SHELL-03 | 默认 shell | 部分 | P0 / S | 每个平台一个默认 shell；Windows 默认从硬编码 `powershell.exe` 改为「目录里评分最高的本地 shell」 | 保留显式覆盖；空值才走默认 |
| SHELL-04 | Git Bash 识别 | 缺 | P0 / S | 识别 `C:\Program Files\Git\bin\bash.exe`，与 `System32\bash.exe` 区分；显示为「Git Bash」而非「bash」 | 查 `HKLM\SOFTWARE\GitForWindows` + `git.exe` 相对路径；排除 System32 |
| SHELL-05 | WSL 发行版 | 缺 | P0 / M | 列出每个发行版（名称、版本、默认标记、运行状态），可单独建 profile；启动用 `wsl.exe -d <D> --cd <dir>` | `HKCU\...\Lxss` 优先；`wsl -l -v` 兜底（UTF-16LE）；不要按本地化表头解析 |
| SHELL-06 | MSYS2 / Cygwin | 缺 | P2 / M | 识别安装根，启动时设置 `MSYSTEM=UCRT64/MINGW64/MSYS` 或 `CHERE_INVOKING=1` | 注册表卸载项 / `Installations` 值；`cygpath` 做路径校验 |
| SHELL-07 | PowerShell 5.1 / 7 差异 | 部分 | P0 / S | 5.1 没有 `-WorkingDirectory`；7 有。两者都用 `-NoLogo -NoExit -ExecutionPolicy Bypass -Command` | 以文件版本或 `$PSVersionTable` 判定，不靠路径猜 |
| SHELL-08 | cmd + Clink | 缺 | P2 / M | 启动 `cmd.exe /K`；若装了 Clink，注入其初始化；不覆盖用户已有的 `AutoRun` | 读 `HKCU\Software\Microsoft\Command Processor\AutoRun`，追加而非替换 |
| SHELL-09 | nu / elvish / xonsh | 缺 | P3 / M | 能发现并启动；nu 用 `--config` 注入集成；其余至少不破坏启动 | 先支持启动，再谈集成 |
| SHELL-10 | 登录 shell / 交互 shell | 部分 | P1 / S | zsh/bash 的 login 与 interactive 语义可选；macOS 默认 login | 现有 `ELY_BASH_LOGIN` 只覆盖 macOS；扩成 profile 选项 |
| SHELL-11 | 提权启动 | 缺 | P2 / M | 明确显示「以管理员运行」；失败时解释 ConPTY 完整性级别限制 | 整个应用提权，或调用 `gsudo` / Win11 `sudo`；不要假装单标签提权 |
| SHELL-12 | 工作目录策略 | 部分 | P0 / S | `Home` / `当前会话 cwd` / `固定目录` / `上次目录` 四选一；新标签默认继承当前 cwd | `StartIn` 扩展；cwd 不存在时按 PATH-05 回退 |
| SHELL-13 | 启动失败诊断 | 缺 | P0 / S | 启动失败时保留 stderr 前几行、退出码、尝试过的路径，并给「换 shell / 编辑 profile / 重试」 | 不要只弹「failed to spawn」；错误要可复制 |
| SHELL-14 | Shell 集成安装/修复 | 部分 | P0 / M | 自动写入集成脚本；提供「重新安装」「查看脚本」「卸载」；修改用户 rcfile 前必须显式同意 | 现有 `install()` 已写脚本；补 UI 与备份 |
| SHELL-15 | 每 profile 环境变量 | 缺 | P1 / S | KEY=VALUE 列表，支持 `${VAR}` 展开、删除、排序；敏感值用掩码 | 只作用于子进程；不写日志 |
| SHELL-16 | 启动耗时 | 缺 | P2 / S | profile 卡片显示最近启动耗时；超过 1s 给提示（WSL 冷启动、Git Bash 跨盘） | 记录 `spawn` 到首帧/首个 OSC 7 |

### 2.2 CONN：SSH、串口、容器与一切远程

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| CONN-01 | SSH config 导入 | 缺 | P0 / M | 枚举 `~/.ssh/config` 的 Host（含 `Include`，跳过通配），生成主机列表；只读，不修改用户文件 | 不要手写完整解析器；`ssh -G` 取生效值 |
| CONN-02 | `ssh -G` 生效配置 | 缺 | P0 / S | 对每个主机解析 `hostname/user/port/identityfile/proxyjump/localforward/remoteforward` | 输出是稳定接口；缓存结果，config mtime 变化时失效 |
| CONN-03 | 主机列表 UX | 缺 | P0 / M | 搜索、最近、收藏、分组、标签；显示 `user@host:port` 与跳板机；空状态给「从 config 导入」 | 左栏新增「远程」区，或复用会话树的分组 |
| CONN-04 | 快速连接 | 缺 | P1 / S | 输入 `user@host:port` 直接连；支持粘贴完整 `ssh` 命令并解析 | 解析失败时原样当命令执行并提示风险 |
| CONN-05 | 认证 | 缺 | P1 / M | 密钥 / agent / 密码 / askpass；密码走系统凭据库 | Windows Credential Manager、macOS Keychain、Linux Secret Service |
| CONN-06 | known_hosts | 缺 | P1 / S | 查看、删除某主机；主机密钥变化时阻断并显示新旧指纹 | `ssh-keygen -F` / `-R`；不要自动接受 |
| CONN-07 | ProxyJump 链 | 缺 | P1 / M | 显示 `本地 → 跳板 → 目标`，失败时指出是哪一跳 | 解析 `ProxyJump` / `ProxyCommand`；`ssh -G` 验证 |
| CONN-08 | 端口转发 | 缺 | P1 / L | 增删 `-L/-R/-D`；活动转发列表、占用端口、启停、复制命令 | 监听进程状态；端口冲突要明确报错 |
| CONN-09 | Agent / X11 转发 | 缺 | P2 / S | profile 开关；显示是否生效 | `ForwardAgent` / `ForwardX11`；Windows X11 需额外说明 |
| CONN-10 | 断线重连 | 缺 | P1 / M | 检测子进程退出与 OSC 133 `D;<exit>`；一键同 profile 重开；可选 `tmux new -A -s ely` | Windows OpenSSH 无 ControlMaster，会话保持靠 tmux/mosh |
| CONN-11 | mosh | 缺 | P2 / M | profile 类型 mosh；远端无 `mosh-server` 时给安装提示 | Windows 侧建议走 WSL |
| CONN-12 | 远程 tmux 管理 | 缺 | P2 / L | 列出/新建/附着/杀死远端会话；可选启动时自动 attach | 解析 `tmux ls`；注意 tmux 会代答 DA1/DECRQM |
| CONN-13 | SFTP 文件浏览 | 缺 | P2 / L | 侧栏文件树；上传/下载/重命名/删除/权限；双击编辑 | 独立 SSH 通道；`russh` 或 `sftp` 子系统 |
| CONN-14 | 串口 | 缺 | P3 / M | 枚举 COM 口；波特率/数据位/校验/停止位/流控；十六进制视图可选 | `HKLM\HARDWARE\DEVICEMAP\SERIALCOMM`；不是 pty，需单独实现 |
| CONN-15 | telnet / raw TCP | 缺 | P3 / M | 基础 TCP 连接；NAWS 协商；明确提示「不是安全协议」 | 与 pty 语义不同，别混进同一抽象 |
| CONN-16 | Docker / K8s exec | 缺 | P2 / M | profile 类型 container：`docker exec -it -w ...`；K8s 选 namespace/pod/container | 必须 `-t`；容器名异步补全 |
| CONN-17 | Dev Container / 远程工作区 | 缺 | P3 / XL | 选择 devcontainer 后进入容器；cwd/文件面板跟随 | 依赖外部 CLI；先做「打开一个命令」的薄封装 |
| CONN-18 | 连接状态徽标 | 缺 | P1 / S | 标签/会话行显示 `WSL: archlinux`、`user@host`、跳板；断开变红 | 数据来自 profile + 子进程状态 |
| CONN-19 | 连接诊断 | 缺 | P2 / S | 复制「实际执行的命令 + 环境变量 + stderr」；提供 `ssh -vvv` 开关 | 一键生成诊断文本，注意脱敏 |
| CONN-20 | 凭据存储 | 缺 | P1 / M | 密码/口令不落明文；可删除；锁定策略跟随系统 | 系统凭据库；配置文件只存引用 |
| CONN-21 | 远程 TERM / 真彩色 | 部分 | P1 / S | 进 SSH 时协商 `COLORTERM`、`TERM_PROGRAM`；tmux 下提示 RGB 配置 | 服务端 `AcceptEnv` 不可控，失败要降级而不是报错 |
| CONN-22 | 会话克隆 | 缺 | P2 / S | 复制一个连接配置开新标签；可「克隆并改端口/用户」 | 不复制运行时状态 |

### 2.3 PATH：cwd、路径与拖放

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| PATH-01 | `Location` 模型 | 缺 | P0 / M | 统一表示 `Local / WSL { distro, path } / SSH { profile, path } / Container` | 所有 cwd 相关 UI 都消费它，不各自解析字符串 |
| PATH-02 | OSC 7 / 9;9 / 1337 | 部分 | P0 / M | 解析 `file://host/path`、ConEmu `9;9`、iTerm2 `CurrentDir`；百分号解码；拒绝非 `file:` | 现有 tap 已处理 OSC 7；补 9;9 与 `Location` |
| PATH-03 | WSL ↔ Windows 映射 | 缺 | P0 / M | `/home/u` ↔ `\\wsl.localhost\D\home\u`；`/mnt/c/x` ↔ `C:\x` | `wslpath` 异步 + 缓存；它不认 `\\wsl.localhost`，只认 `\\wsl$` |
| PATH-04 | Git Bash / Cygwin 映射 | 缺 | P0 / S | `/c/x` ↔ `C:\x`；`/usr/bin` 不映射到安装根 | 纯字符串映射 + `cygpath` 校验；映射不上落 home |
| PATH-05 | cwd 校验与回退 | 缺 | P0 / S | 顺序：profile.cwd → `%USERPROFILE%` → `C:\`；Unix：`$HOME` → `/` | WSL/SSH 的 cwd 不在本机，不做 `metadata` |
| PATH-06 | UNC 处理 | 缺 | P1 / S | cmd 不能以 UNC 为 cwd；改为 `%SystemRoot%` + 启动后 `pushd` | 明确提示，不要静默失败 |
| PATH-07 | 拖放路径转换 | 部分 | P0 / M | 按目标转换 + 引用：WSL `wslpath -u`、Git Bash `cygpath -u`、本地按 `Quote`、SSH 不转换 | 现有 `Quote` 可用；补变换层；一律括号粘贴 |
| PATH-08 | 复制路径格式 | 缺 | P1 / S | 复制为 Windows / POSIX / WSL / 相对 / 带引号 / URI | 选择菜单或命令面板；路径来自选区/块 |
| PATH-09 | 打开所在位置 | 缺 | P1 / S | 「在资源管理器/Finder 中显示」「在编辑器中打开」；远程路径提示不可用 | `explorer /select,`、`open -R`、`xdg-open`；编辑器命令可配 |
| PATH-10 | `file:line:col` 链接 | 缺 | P1 / M | 输出里的 `src/main.rs:42:7` 可点击，交给编辑器跳转；Windows 盘符与 Unix 路径都识别 | 正则 + 存在性校验；`code -g file:line:col` |
| PATH-11 | 目录面包屑 | 缺 | P2 / S | 窗格顶部显示 cwd，可点任意一级；远程显示 badge | 数据来自 `Location`；不要自己拼字符串 |
| PATH-12 | 最近目录 | 缺 | P2 / S | 新标签/快速切换列出最近 cwd；可固定 | 本地记录，按会话清理 |
| PATH-13 | 远程路径动作 | 缺 | P2 / S | 远程 cwd 的「打开文件」走 SFTP 或提示；不要错误地调用本地资源管理器 | 由 `Location` 分支决定 |

### 2.4 TAB：标签、窗格、窗口与工作区

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| TAB-01 | 树形标签完善 | 部分 | P0 / M | 左栏一行一个标签，展开显示其窗格；这是本项目的特色，保留 | 现有 `tabs_column`；补折叠/过滤/拖动 |
| TAB-02 | 顶部标签条（可选） | 缺 | P3 / M | 若做，必须是同一 `tabs: Vec<Tab>` 的第二种视图；默认关闭 | 不要引入第二套模型；先做完整树形标签 |
| TAB-03 | 标签拖拽排序 | 缺 | P1 / M | 左栏内上下拖动，插入指示线；快捷键「上移/下移」 | `Vec<Tab>` 重排 + 持久化 |
| TAB-04 | 固定 / 颜色 / 图标 | 缺 | P2 / S | 固定标签不被误关；颜色标识环境（prod 红、staging 黄）；图标来自 profile | 固定项排在前面；颜色贯穿标签行与边框 |
| TAB-05 | 标签重命名 | 部分 | P1 / S | 会话可重命名（已有）；标签名独立于会话名；清空回退到焦点窗格标题 | `Tab.name` 已有；补双击标签重命名 |
| TAB-06 | 悬停预览 | 缺 | P3 / M | 悬停标签显示最后几行/缩略图；不切换标签 | 需要离屏渲染或文本快照；注意性能 |
| TAB-07 | 标签总览 | 缺 | P3 / L | 类似 Mission Control 的网格，快速跳转；适合 10+ 标签 | 离屏快照 + 键盘导航 |
| TAB-08 | MRU 切换 | 部分 | P1 / S | `Ctrl+Tab` 按最近使用而不是顺序；顺序切换保留为另一个键 | 现有按 `session_order` 顺序；加 `last_used` 时间戳 |
| TAB-09 | 标签组 / 工作区 | 缺 | P2 / L | 按项目分组，组可折叠；工作区 = 根目录 + profile + 布局 | 组是标签的上层；不要和「窗口」混淆 |
| TAB-10 | 未读 / 失败 / 等待角标 | 部分 | P1 / M | 标签行显示：有输出未看、命令失败、等待输入、运行中 | 消费 OSC 133 `D;exit`、OSC 7501、焦点状态 |
| TAB-11 | 进度条 | 部分 | P1 / S | 标签/会话行细进度条；错误红、暂停黄、不确定条纹 | OSC 9;4 已有；补 UI 与超时清除 |
| TAB-12 | 关闭确认 | 缺 | P1 / S | 有前台进程/未保存任务时确认；可「不再询问」，可在设置恢复 | 检查 PTY 前台进程组/子进程；批量关闭要汇总 |
| TAB-13 | 克隆标签/会话 | 缺 | P2 / S | 用同一 profile、同一 cwd 开新标签；不复制滚动缓冲 | 与 SHELL-12 继承 cwd 复用 |
| TAB-14 | 移到新窗口 | 缺 | P2 / M | 把一个标签拖出成新窗口；反向也能合并 | 需要多窗口模型与 PTY 迁移（或重建 + 重放命令） |
| TAB-15 | 分屏比例拖动 | 缺 | P1 / M | 拖分隔条改比例；双击均分；比例持久化到会话恢复 | `Pane.width/height` 已是十分位；改成可存储的精确比例 |
| TAB-16 | 窗格放大 | 缺 | P1 / S | `toggle_zoom` 最大化当前窗格，快捷键同 Ghostty；再按还原 | 只影响绘制，不改变布局树；状态栏提示 |
| TAB-17 | 窗格交换 / 移动 | 缺 | P2 / M | 与相邻窗格交换；拖到另一个标签 | 交换 session id；移动时保持 PTY 运行 |
| TAB-18 | 窗格标题 / 焦点环 | 部分 | P1 / S | 顶部细条显示标题/cwd/连接；非焦点窗格边缘压暗 | 不要每窗格都画重标题栏；保持终端面积 |
| TAB-19 | 广播输入 | 缺 | P2 / M | 输入同时发给选定窗格；显著警告条 + 一键退出 | 白名单选择；默认关闭；不要在密码提示时误开 |
| TAB-20 | 同步滚动 | 缺 | P3 / S | 多窗格一起滚动，用于对比日志；各自滚动时自动退出 | 只同步用户滚动，不同步程序输出 |
| TAB-21 | 布局预设 | 缺 | P2 / M | 保存/恢复「2×2」「左大右小」等布局；按工作区自动应用 | 序列化布局树；与 TAB-09 共用 |
| TAB-22 | 多窗口 | 缺 | P2 / L | 新建窗口、窗口间移动标签、每窗口独立焦点 | GPUI `open_window`；会话/设置共享 |
| TAB-23 | 快速终端 | 缺 | P2 / L | 全局热键呼出无边框置顶终端，失焦隐藏，跟随当前 cwd | macOS NSPanel、Win32 HWND、Wayland layer-shell |
| TAB-24 | 画中画 / 迷你窗 | 缺 | P3 / M | 把一个会话缩成小窗常驻，看构建/日志 | 独立窗口 + 只读/可输入切换 |
| TAB-25 | 全屏 / 禅模式 | 部分 | P1 / S | 隐藏左栏、右栏、标题栏，只留终端；`Esc` 退出 | 复用现有列开关，加「一键全隐」 |
| TAB-26 | 窗口几何记忆 | 部分 | P1 / S | 记住大小/位置/最大化；多显示器变化时回退到主屏 | 设置已有宽高；补位置与屏幕 id |
| TAB-27 | 焦点跟随鼠标 | 缺 | P2 / S | 可选：鼠标移入窗格即聚焦；默认关闭 | 与点击移动光标冲突，需分开 |
| TAB-28 | 拖到边缘分屏 | 缺 | P2 / M | 拖文件/标签到窗格边缘，出现分屏预览 | 与 TAB-15 共用布局逻辑 |
| TAB-29 | 标签搜索 / 溢出 | 缺 | P2 / S | 标签多时提供过滤框；左栏可滚动、可折叠 | 过滤只影响显示，不改变顺序 |
| TAB-30 | 会话树折叠 / 过滤 | 部分 | P1 / S | 折叠标签下的窗格；按名称/连接过滤 | 现有树形行；补折叠状态持久化 |

### 2.5 INP：键盘、输入法、鼠标

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| INP-01 | legacy / kitty 编码 | 已有 | — | xterm 编码、kitty flags 1/2/4/8/16、win32-input-mode | 保持回归测试 |
| INP-02 | modifyOtherKeys | 缺 | P1 / M | 程序请求 `CSI > 4;n m` 后，Alt/Ctrl 组合按 xterm 方式编码 | vte 能解析但 handler 为空；在 `keys.rs` 接上 |
| INP-03 | Option as Alt | 已有 | — | macOS Option 键四种模式 | 保持 |
| INP-04 | IME | 部分 | P0 / M | 中文/日文/韩文组合；候选窗贴光标；组合下划线；提交/取消不丢字符 | GPUI IME 事件与单元格位置对齐；CJK 宽字符边界 |
| INP-05 | 死键 / Compose | 缺 | P2 / S | 支持死键序列与 compose 键；输入法冲突时让 IME 优先 | 平台键盘布局差异大，先 Windows/macOS |
| INP-06 | 键盘布局 / 重映射 | 缺 | P3 / M | 物理键 → 逻辑键重映射；可用于 Caps→Ctrl | 读取平台布局；不要自己猜扫描码 |
| INP-07 | 快捷键与终端透传 | 部分 | P1 / M | 明确哪些组合归终端、哪些发给程序；`Ctrl+Shift+*` 约定；可配置「强制透传」 | 冲突检测；不要吞掉 shell 常用键 |
| INP-08 | 键序列宏 | 缺 | P2 / M | 一个快捷键发送文本/十六进制/多步按键，可带延迟 | 显式配置；用于 `Esc` 序列、登录流程；不要录制密码 |
| INP-09 | 粘贴管线 | 部分 | P0 / S | 括号粘贴、剥 ESC/C0、多行确认、大粘贴限速、粘贴预览 | 现有基础可用；补预览与危险内容确认 |
| INP-10 | 鼠标编码 | 部分 | P1 / S | 9/1000/1002/1003/1006/1005 已有；补 1015/1016 与滚轮 64–67 | 1016 报像素坐标；注意与 1006 互斥 |
| INP-11 | 点击移动光标 | 已有 | — | 按 OSC 133 判断 shell 正在读行 | 保持；补「只在安全时」 |
| INP-12 | 触控板 / 触屏 | 部分 | P2 / M | 双指滚动、捏合缩放字体、触屏长按选择 | 平台事件不同；捏合只改 UI 字号 |
| INP-13 | 安全键盘输入 | 缺 | P2 / M | 开启后阻止第三方输入监控（平台能力允许时）；密码场景提示 | macOS `EnableSecureEventInput`；其他平台说明限制 |
| INP-14 | 快捷键速查 | 缺 | P1 / S | `Ctrl+Shift+/` 弹出当前上下文快捷键；可搜索 | 与命令面板共用 action 表 |
| INP-15 | 按键调试器 | 缺 | P3 / S | 显示当前按键的扫描码/键码/编码结果，用于排查 IME/键盘问题 | 设置里的诊断页；默认隐藏 |
| INP-16 | CJK 标点 / 全角 | 部分 | P1 / S | 中文输入法下全角标点宽度正确；不把中文标点当控制字符 | 宽度表 + 字素簇；与 INP-04 一起测 |
| INP-17 | 键盘协议查询 | 已有 | — | `CSI ? u` 等 | 保持 |

### 2.6 SEL：选择与剪贴板

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| SEL-01 | 字符/词/行/矩形选择 | 部分 | P1 / M | 拖选、双击词、三击行、Alt 矩形；Shift 扩展 | 矩形选择要处理宽字符与换行 |
| SEL-02 | 智能选词 | 缺 | P1 / M | 双击自动识别 URL、路径、`file:line`、IP、端口、git SHA、括号内内容 | 一组正则 + 边界修正；可配置 |
| SEL-03 | 选择动作菜单 | 缺 | P1 / M | 选中后右键/浮条：复制、搜索、打开链接、在编辑器中打开、复制路径、发送到窗格 | 动作按内容类型启用；不要堆一长串 |
| SEL-04 | 复制为 | 缺 | P2 / S | 纯文本、带引号、转义、Markdown 代码块、HTML、URI | 复用 `Quote` 与路径格式 |
| SEL-05 | 复制即选 | 已有 | — | 设置开关 | 保持 |
| SEL-06 | 剪贴板历史 | 缺 | P2 / M | 最近 20–50 条，搜索、固定、删除、粘贴；敏感内容不记录 | 本地内存/加密存储；退出可清空 |
| SEL-07 | 粘贴预览 | 缺 | P1 / S | 多行/含控制字符/超长时先显示预览与「将粘贴 N 行」 | 阻止 `rm -rf` 类误粘贴；可「仍然粘贴」 |
| SEL-08 | OSC 52 策略 | 部分 | P0 / S | 写入可用；读取默认关闭；写入限长、大内容确认 | 现有读取已拒绝；补长度与确认 |
| SEL-09 | 命令块选择 | 缺 | P1 / S | 选中整条命令/输出，而不是逐行拖 | 依赖 BLOCK-01 |
| SEL-10 | 选区保持 | 部分 | P2 / S | 滚动、输出刷新、切换焦点后选区仍在；新输出不清除选区 | 记录 anchor/head 的逻辑坐标 |
| SEL-11 | 拖出文本 | 缺 | P3 / M | 把选区拖到编辑器/浏览器；平台拖放 | 文本 + 纯文本 MIME；注意安全 |
| SEL-12 | 复制路径/URL/哈希 | 缺 | P1 / S | 一键复制当前目录、选中路径、URL、commit SHA | 与 PATH-08 共用格式化 |

### 2.7 FIND：搜索、跳转与标记

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| FIND-01 | 搜索栏 | 已有 | — | 增量搜索、滚动缓冲 | 保持 |
| FIND-02 | 搜索选项 | 部分 | P1 / S | 大小写、整词、正则三个开关；显示当前模式 | `find.rs` 已有正则；补 UI 与状态 |
| FIND-03 | 搜索选区 | 缺 | P1 / S | 有选区时打开搜索并预填，可选择「仅选区内」 | 选区逻辑坐标 |
| FIND-04 | 跨标签搜索 | 缺 | P2 / L | 在所有标签/窗格搜索，结果按会话分组；点击跳转 | 后台线程；限制并发 |
| FIND-05 | 结果计数/导航 | 部分 | P1 / S | `3/17`、上一个/下一个、循环、Esc 回底 | 现有查找需补 UI 状态 |
| FIND-06 | 滚动条命中标记 | 缺 | P1 / M | 滚动条上显示命中位置；命令成功/失败也用不同颜色 | 与 BLOCK-06 共用标记层 |
| FIND-07 | 仅搜索命令/输出 | 缺 | P2 / M | 按 OSC 133 块过滤，只在命令或输出里搜 | 依赖 BLOCK-01 |
| FIND-08 | 跳转提示符/命令/错误 | 部分 | P1 / S | 上一个/下一个 prompt、命令、失败命令；快捷键同 Ghostty | 现有 marks 已有 A/C/D；补失败过滤 |
| FIND-09 | 书签 / 注解 | 缺 | P2 / M | 给某行加书签/备注；书签列表可跳转；可导出 | 行标记要随滚动缓冲淘汰 |
| FIND-10 | 快速选择 / hints | 缺 | P2 / L | 键盘触发，给屏幕上的 URL/路径/哈希打标签，按键跳转/复制 | 类似 Vimium；标签不能遮住内容太久 |
| FIND-11 | 命令历史模糊搜索 | 缺 | P2 / M | 从 OSC 133 收集命令，`Ctrl+R` 风格模糊搜索；可插入/重跑 | 终端侧索引；不替代 shell 历史 |
| FIND-12 | 搜索性能 | 部分 | P1 / M | 100k 行首命中 < 50ms；搜索期间 UI 不卡；可取消 | 每行纯文本镜像 + 后台线程 |
| FIND-13 | 正则错误提示 | 缺 | P2 / S | 非法正则就地显示错误位置，不静默失败 | 复用 regex 错误信息 |

### 2.8 BLOCK：命令块与 shell 集成

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| BLOCK-01 | 命令块模型 | 部分 | P0 / M | 每条命令 = prompt + 命令行 + 输出 + 退出码 + 时长；由 OSC 133 A/B/C/D 切分 | 现有 `marks.rs` 有 A/N/B/I/C/D；补数据结构与生命周期 |
| BLOCK-02 | 折叠输出 | 缺 | P1 / M | 折叠一条命令的输出，只留命令行与状态；展开恢复 | 折叠是视图状态，不改变网格 |
| BLOCK-03 | 复制命令/输出 | 缺 | P1 / S | 分开复制命令行与输出；输出保留纯文本 | 依赖 BLOCK-01 |
| BLOCK-04 | 重跑命令 | 缺 | P1 / S | 点击/快捷键重跑当前块；可选 `sudo` 前缀；重跑前确认 | 把命令写到 PTY，不自动回车 unless 确认 |
| BLOCK-05 | 块导航 | 缺 | P1 / S | 上一条/下一条/上一条失败；滚动到块首 | 复用 FIND-08 的标记 |
| BLOCK-06 | 块 gutter / 状态色 | 缺 | P1 / M | 左侧细条：运行中、成功、失败、被中断；hover 显示时长/退出码 | 与滚动条标记同源 |
| BLOCK-07 | 时长 / 退出码 | 部分 | P1 / S | 显示 `1.2s`、`exit 1`；长任务显示运行中 | 从 C/D 的时间戳算；不要伪造 |
| BLOCK-08 | 失败诊断 | 缺 | P2 / M | 失败块给「查看 stderr」「复制错误」「搜索相似错误」「解释」 | 不自动执行修复 |
| BLOCK-09 | 块内搜索 | 缺 | P2 / S | 只在当前块内查找 | 与 FIND-07 共用 |
| BLOCK-10 | 导出块 | 缺 | P2 / S | 导出为文本/Markdown/JSON；可只导出失败块 | 含命令、输出、退出码、时长 |
| BLOCK-11 | 命令历史索引 | 缺 | P2 / M | 按会话/目录/退出码索引命令；可搜索、统计最慢/最常失败 | 只存命令与元数据，不存完整输出 |
| BLOCK-12 | 两次运行 diff | 缺 | P3 / L | 同一命令两次输出并排 diff，用于回归/构建对比 | 需要输出快照与对齐算法 |
| BLOCK-13 | 分享为 Markdown | 缺 | P3 / S | 生成带命令、输出、退出码的 Markdown 片段 | 注意脱敏路径/密钥 |
| BLOCK-14 | 交给 AI | 缺 | P2 / S | 选中块 →「解释错误」「给出修复建议」；只发送该块，不发送整屏 | 显式触发；预览发送内容 |
| BLOCK-15 | OSC 633 兼容 | 缺 | P2 / S | 兼容 VS Code 的 `633;A/B/C/D` 与 `D;exit` 写法 | 与 133 统一解析 |
| BLOCK-16 | 无集成降级 | 部分 | P1 / S | 没有 shell 集成的会话，块功能隐藏并提示「安装集成」 | 不要显示空 gutter |

### 2.9 STAT：通知、进度与状态

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| STAT-01 | 通知 | 已有 | — | OSC 9 / 777 / 99、系统通知、限流 | 保持 |
| STAT-02 | 通知聚合 | 缺 | P1 / M | 同一会话/同一 agent 的通知合并；「还有 3 条」 | 按会话 + 时间窗聚合 |
| STAT-03 | 勿扰 / 专注 | 缺 | P1 / S | 一键静音；可定时；全屏时自动 | 状态在标题栏可见 |
| STAT-04 | 任务栏集成 | 缺 | P1 / M | 进度、错误角标、闪烁/overlay；点击回到会话 | Windows `ITaskbarList3`；macOS dock badge |
| STAT-05 | 会话状态点 | 部分 | P1 / S | 运行中/空闲/等待输入/失败 | 数据来自 7501 + 133 + 焦点 |
| STAT-06 | 进度条 | 部分 | P1 / S | OSC 9;4 的 0–4 状态；超时清除 | 现有解析；补 UI |
| STAT-07 | OSC 7501 | 已有 | — | 状态记录、查询回复、记录树 | 领先多数终端，保持并产品化 |
| STAT-08 | Agent 状态 | 部分 | P1 / M | working / blocked / idle / error；blocked 优先提醒 | 与 STAT-05 统一状态机 |
| STAT-09 | 等待输入提醒 | 缺 | P1 / M | agent 等待审批/输入超过 N 秒：角标 + 通知 + 声音（可关） | 不要每秒弹；升级策略 |
| STAT-10 | 长任务完成 | 缺 | P1 / S | 超过阈值（如 10s）的命令完成时通知；失败优先 | 从 BLOCK-01 的时长算 |
| STAT-11 | 点击跳转 | 缺 | P1 / S | 通知/角标点击定位到会话、窗格、命令块 | 需要稳定的 session id |
| STAT-12 | 自定义钩子 | 缺 | P3 / M | 命令完成/失败/通知时执行本地命令或 webhook；默认关闭 | 明确安全警告；不要把输出当 shell |
| STAT-13 | Bell | 部分 | P1 / S | 视觉闪烁、声音、标签角标；可分别开关 | 不要默认响铃 |
| STAT-14 | 状态栏 | 缺 | P2 / S | 显示连接、TERM、编码、尺寸、前台进程、Git 分支 | 可隐藏；不要永久占一行 |
| STAT-15 | 前台进程 | 缺 | P2 / M | 显示当前前台进程名/PID；可结束进程（确认） | Unix `tcgetpgrp` 已有；Windows 用 sysinfo/ConPTY |
| STAT-16 | 监听端口 | 缺 | P3 / M | 显示当前会话相关端口，可点击打开浏览器；来自显式配置或系统查询 | 不做全机扫描；隐私与性能都要考虑 |
| STAT-17 | Git 分支徽标 | 缺 | P2 / S | 窗格/状态栏显示分支、脏标记 | 读 `.git/HEAD` 或复用 Git 面板 |
| STAT-18 | 会话健康 | 缺 | P2 / S | PTY 长时间无输出/无响应、子进程退出未清理 | 只提示，不自动杀 |
| STAT-19 | 失败列表 | 缺 | P2 / S | 本次会话所有失败命令的列表，可跳转/重跑 | 依赖 BLOCK-11 |

### 2.10 UI：外观、主题、字体

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| UI-01 | 内置主题 | 已有 | — | 14 套 + 自定义颜色 | 保持 |
| UI-02 | 主题导入/导出 | 缺 | P1 / M | 导入 iTerm2 `.itermcolors`、Windows Terminal JSON、Ghostty、base16；导出当前主题 | 解析后落到统一 `Colors`；导入前预览 |
| UI-03 | 主题预览/轮换 | 缺 | P2 / S | 设置里实时预览；可选每日/随机 | 预览不改全局，确认才应用 |
| UI-04 | 明暗跟随系统 | 已有 | — | `dark_theme` | 保持；补日出日落可选 |
| UI-05 | 每 profile 主题 | 缺 | P2 / S | 不同环境不同配色（prod 红、staging 黄） | profile 覆盖全局 |
| UI-06 | 背景图 / 模糊 / 透明度 | 部分 | P1 / M | 图片、模糊、透明度、暗化；按 profile | 现有 opacity/blur；补图片与性能限制 |
| UI-07 | 平台材质 | 部分 | P2 / M | Windows Mica/Acrylic、macOS vibrancy、Wayland blur | 现有 4 平台 blur；补材质选择与降级 |
| UI-08 | 字体 / 回退 / 连字 | 部分 | P1 / S | 主字体、回退链、连字开关、粗/斜体独立 | 现有 `font_fallbacks`、`ligatures`；补 UI 与验证 |
| UI-09 | 可变字体 / 特性 | 缺 | P2 / M | wght/opsz 轴、字距、`font-feature-settings` | 需要字体解析；不要假设所有字体支持 |
| UI-10 | Emoji 彩色 | 部分 | P1 / M | Segoe UI Emoji / Apple Color Emoji 的 COLR/CBDT；ZWJ 序列 | 与 A11Y-11、PROTO 字素一起 |
| UI-11 | 光标 | 部分 | P1 / S | 块/竖线/下划线/空心、闪烁、颜色、失焦样式 | 已有样式与闪烁；补颜色/失焦 |
| UI-12 | 选区/搜索高亮 | 部分 | P1 / S | 选区、搜索命中、当前命中、非焦点选区分别配色 | 对比度检查 |
| UI-13 | 下划线样式 | 部分 | P1 / S | 单/双/波浪/点线/虚线 + 彩色下划线 | alacritty 有 SGR；补渲染与主题默认 |
| UI-14 | 加粗/暗淡/最小对比度 | 部分 | P1 / S | bold-as-bright 可选；dim 强度；最小对比度 | 主题可覆盖 |
| UI-15 | 调色板覆盖 | 缺 | P2 / S | ANSI 16 色可单独改；导入主题后微调 | 与主题模型统一 |
| UI-16 | 标签栏样式 | 缺 | P2 / S | 树形/顶部/隐藏；密度、图标、颜色 | 与 TAB-02 绑定 |
| UI-17 | 窗口标题格式 | 缺 | P2 / S | `{title} — {cwd} — {profile}` 模板；可隐藏 | 标题清洗后使用 |
| UI-18 | 内边距 / 圆角 / 边框 | 部分 | P2 / S | 每 profile padding；窗口圆角；焦点边框 | 现有 padding；注意与分屏边框冲突 |
| UI-19 | 动画 / 减少动态 | 部分 | P1 / S | 面板滑入、标签切换、宠物动画；遵循系统「减少动态」 | 不要为动画掉帧 |
| UI-20 | 透明度快捷键 | 缺 | P3 / S | 临时调透明度，便于看背后内容 | 不写入配置除非固定 |
| UI-21 | 主题截图 | 缺 | P3 / S | 导出带窗口背景的主题预览图 | 用于分享；不含终端内容或先确认 |
| UI-22 | Ely 外观 | 已有 | — | 宠物位置、开关、情绪 | 见 FUN 区 |
| UI-23 | 字体渲染调优 | 缺 | P2 / M | gamma/hinting/subpixel、LCD 与否；平台默认合理 | 不要暴露过多旋钮 |
| UI-24 | 高对比度 / 色盲 | 缺 | P2 / M | 高对比主题、色盲友好调色板、不仅靠颜色传达状态 | 与 A11Y-02 共用 |
| UI-25 | 安全区 | 缺 | P2 / S | macOS 刘海/圆角、Windows 标题栏按钮避让 | 平台窗口 API |
| UI-26 | 滚动条样式 | 部分 | P2 / S | 自动隐藏、宽度、颜色、标记 | 现有 scrollbar；补样式 |
| UI-27 | 平滑滚动 / 光标拖尾 | 缺 | P3 / S | 可选、默认关；高刷屏才开 | 可能增加延迟，需基准 |

### 2.11 CFG：配置、键位、profile

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| CFG-01 | TOML 配置 | 缺 | P0 / M | `config.toml`，可读、可注释、可版本控制；设置窗口仍可用 | `toml_edit` 保留注释；schema 版本 |
| CFG-02 | 热重载 | 缺 | P0 / M | 保存即生效；debounce 200ms；坏配置保留旧值并提示行号 | 监听 mtime/hash；原子替换 |
| CFG-03 | 设置窗口双向同步 | 缺 | P0 / M | 设置窗口改文件；文件改设置窗口；冲突时文件优先并标注 | 同一模型两个视图；避免回写循环 |
| CFG-04 | schema / 迁移 | 缺 | P0 / S | 旧 `settings.json` 自动迁移；未知字段警告；版本升级脚本 | 不要 `deny_unknown_fields` 直接拒绝用户配置 |
| CFG-05 | include / 分层 | 缺 | P2 / M | 主配置 include 片段；全局 → 主机 → 项目 | 循环 include 检测 |
| CFG-06 | 环境变量插值 | 缺 | P2 / S | `cwd = "${HOME}/src"`；未定义时给错误 | 只做一层；不要执行命令 |
| CFG-07 | 条件配置 | 缺 | P2 / M | 按 OS/hostname/环境切换 profile/主题 | 条件求值在启动时，结果可查看 |
| CFG-08 | profile 继承 | 缺 | P1 / M | `base` profile + 覆盖；避免复制十几份 | 明确合并规则（标量覆盖、列表追加/替换） |
| CFG-09 | 键位自定义 | 缺 | P0 / M | 录制、清除、冲突检测、按平台默认；显示当前绑定 | action 表 + 用户覆盖；与 INP-07 共用 |
| CFG-10 | 配置导入 | 缺 | P2 / M | 从 Ghostty/Windows Terminal/iTerm2/VS Code 导入键位与配色 | 只导入能映射的；列出跳过项 |
| CFG-11 | 配置导出/分享 | 缺 | P3 / S | 导出最小配置片段；可带注释 | 脱敏路径/密钥 |
| CFG-12 | 设置搜索 | 缺 | P1 / S | 搜索设置项、跳转、显示当前值来源（默认/文件/窗口） | 与命令面板共用索引 |
| CFG-13 | 重置 | 缺 | P2 / S | 单项/分组/全部重置；重置前确认；可撤销 | 不要静默覆盖用户文件 |
| CFG-14 | 错误定位 | 缺 | P1 / S | 配置错误显示文件、行、列、期望类型；可点击打开 | `serde_path_to_error` 已在依赖里 |
| CFG-15 | `--config` 覆盖 | 缺 | P3 / S | 启动时指定另一份配置，便于测试 | 不写入默认路径 |
| CFG-16 | 生成带注释的默认配置 | 缺 | P2 / S | 首次运行生成 `config.toml`，每项有注释与默认值 | 不要覆盖已有文件 |

### 2.12 PANEL：文件与 Git 面板协同

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| PANEL-01 | 文件树跟随 cwd | 已有 | — | 会话 cwd 变化时刷新 | 保持；补远程分支 |
| PANEL-02 | 文件搜索/过滤 | 部分 | P1 / M | 按名称过滤、模糊搜索、最近文件 | 大仓库要异步/限制深度 |
| PANEL-03 | Git 状态/分支 | 已有 | — | 分支、变更、最近提交 | 保持 |
| PANEL-04 | Diff | 已有 | — | unified/side-by-side | 保持；补大文件/二进制提示 |
| PANEL-05 | Stage / commit | 已有 | — | 暂存、提交 | 保持；补提交信息模板 |
| PANEL-06 | 在编辑器中打开 | 缺 | P1 / S | 双击文件/右键「在编辑器中打开」；编辑器命令可配 | `code -g`、`zed`、`$EDITOR` |
| PANEL-07 | 拖文件到终端 | 部分 | P1 / S | 从面板拖到终端插入路径；按 PATH-07 转换 | 与拖放管线共用 |
| PANEL-08 | 终端路径 → 面板 | 缺 | P1 / M | 终端里点击文件路径，面板定位/预览该文件 | PATH-10 + PANEL-01 |
| PANEL-09 | 复制路径 | 缺 | P1 / S | 相对/绝对/带引号/URI | 与 PATH-08 共用 |
| PANEL-10 | 在文件管理器中显示 | 缺 | P1 / S | Explorer/Finder/xdg-open | 平台 API |
| PANEL-11 | 最近文件 | 缺 | P2 / S | 当前仓库最近修改/打开的文件 | 本地记录 |
| PANEL-12 | 面包屑 | 部分 | P2 / S | 路径层级可点；远程显示 badge | 现有标签可能已有；补交互 |
| PANEL-13 | Blame | 缺 | P3 / M | 行级作者/提交；点击看 diff | `git blame --porcelain`；大文件限制 |
| PANEL-14 | 冲突解决 | 缺 | P3 / L | 标记冲突块，选择 ours/theirs/both；与 diff 联动 | 不要自建完整 merge 编辑器 |
| PANEL-15 | 大仓库性能 | 部分 | P1 / M | 10 万文件不卡；忽略 `target/`、`node_modules/` | 懒加载 + `.gitignore` |
| PANEL-16 | 多根工作区 | 缺 | P3 / L | 多个根目录并排；每根独立 Git | 与 TAB-09 工作区模型统一 |

### 2.13 XFER：文件传输与远程编辑

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| XFER-01 | SFTP 浏览 | 缺 | P2 / L | 远程文件树、面包屑、权限、大小、修改时间 | `sftp` 子系统；缓存目录列表 |
| XFER-02 | 上传/下载 | 缺 | P2 / L | 拖放上传/下载；进度、取消、失败重试 | 传输队列；不要阻塞 UI |
| XFER-03 | 远程编辑 | 缺 | P2 / L | 双击打开临时副本，保存时上传；mtime 冲突给 diff | 临时目录私有；退出清理 |
| XFER-04 | Zmodem | 缺 | P3 / M | 识别 `rz`/`sz`，自动进入传输模式 | 需要 ZMODEM 状态机；安全提示 |
| XFER-05 | scp/rsync 命令生成 | 缺 | P2 / S | 选中文件生成可复制命令 | 只生成，不自动执行 |
| XFER-06 | 传输队列 | 缺 | P2 / M | 全局队列，显示来源/目标/速度/剩余 | 与通知/任务栏联动 |
| XFER-07 | 断点续传 | 缺 | P3 / M | 大文件失败可续传 | rsync/SFTP 偏移 |
| XFER-08 | 权限/时间戳 | 缺 | P3 / S | 保留/修改权限与 mtime | 明确默认策略 |
| XFER-09 | 本地/远程映射 | 缺 | P2 / M | 远程路径映射到本地临时目录，供编辑器/面板使用 | 与 PATH-01 统一 |
| XFER-10 | 传输安全 | 缺 | P2 / S | 主机密钥校验、路径穿越防护、符号链接策略 | 默认拒绝写到预期根之外 |

### 2.14 AUTO：启动命令、snippets、触发器

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| AUTO-01 | 启动命令 | 缺 | P2 / S | profile 可配置启动时执行的命令列表（如 `cd`、`nvm use`） | 写到 PTY；不要在命令行里拼密码 |
| AUTO-02 | 延迟 / 等待提示符 | 缺 | P2 / S | 等首个 OSC 133 A 或超时后再发下一条 | 不要固定 sleep；失败要停止并提示 |
| AUTO-03 | Snippets | 缺 | P2 / M | 命名片段，插入到光标；支持多行、变量、快捷键 | 显式插入，不自动执行 |
| AUTO-04 | Snippet 变量 | 缺 | P2 / S | `${cwd}`、`${date}`、`${selection}`、`${clipboard}`、`${host}` | 剪贴板变量要确认 |
| AUTO-05 | Snippet 管理 | 缺 | P2 / S | 列表、搜索、分组、导入/导出 | 与命令面板共用 |
| AUTO-06 | 触发器 | 缺 | P3 / M | 输出匹配正则 → 高亮/标记/通知/声音；可选执行命令（确认） | 默认只做视觉；执行输入要显式开启 |
| AUTO-07 | 关键词高亮 | 缺 | P2 / S | `ERROR`、`WARN`、URL、IP、端口、TODO 等规则可配 | 只影响显示，不改输出 |
| AUTO-08 | 命令收藏 | 缺 | P2 / S | 收藏常用命令，按目录/项目分组 | 与 BLOCK-11 区分：这是手动收藏 |
| AUTO-09 | 任务模板 | 缺 | P3 / M | 「Rust 开发」「SSH 运维」一键开一组标签/窗格/命令 | 与 TAB-21 布局预设共用 |
| AUTO-10 | 每目录启动脚本 | 缺 | P3 / S | 进入某目录时提示/执行项目脚本（需确认） | 安全风险高；默认只提示 |
| AUTO-11 | 外部事件钩子 | 缺 | P3 / M | 命令完成/失败/会话退出时调用本地脚本 | 默认关闭；超时与输出限制 |
| AUTO-12 | 发送选区到窗格 | 缺 | P2 / S | 把选中的文本插入另一个窗格或编辑器 | 不自动回车 |
| AUTO-13 | 宏录制 | 缺 | P3 / M | 录制按键序列并回放；用于重复操作 | 不记录密码输入；明确红点状态 |

### 2.15 AI：Agent 时代的终端

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| AI-01 | Agent 状态 | 部分 | P1 / M | working/blocked/idle/error；显示当前命令与耗时 | OSC 7501 已有；统一状态机 |
| AI-02 | 审批 / 等待队列 | 缺 | P1 / M | blocked 的会话排到前面；显示「等待你输入」；一键跳转 | 通知升级策略；不要刷屏 |
| AI-03 | 多 Agent 看板 | 缺 | P2 / L | 所有 agent 会话的网格：状态、任务、耗时、成本 | 只读聚合；点击回到终端 |
| AI-04 | 成本/用量 | 缺 | P3 / S | 由 agent 通过 OSC 上报，终端只显示 | 不抓屏解析；不做计费 |
| AI-05 | 解释错误 | 缺 | P2 / M | 选中失败块 → 发送该块到外部 CLI/模型；返回只读结果 | 显式触发；预览发送内容 |
| AI-06 | 摘要输出 | 缺 | P3 / M | 长输出摘要；只发送选区/块 | 默认关闭；本地模型可选 |
| AI-07 | 建议修复 | 缺 | P2 / S | 给出命令建议，只复制不执行 | 永不自动执行 |
| AI-08 | Diff 审阅 | 缺 | P3 / M | agent 改动的 diff 交给 Git 面板；终端只显示入口 | 不在终端里另造 diff UI |
| AI-09 | 会话命名/颜色 | 缺 | P2 / S | 按任务/agent 命名与配色；自动建议名字（可关） | 本地规则优先 |
| AI-10 | 通知聚合 | 缺 | P1 / S | 多 agent 通知合并；blocked 优先 | 与 STAT-02 共用 |
| AI-11 | 外部 CLI 接口 | 缺 | P2 / M | `elyterminal agent status ...` 或 IPC，让 agent 上报状态 | 文档化协议；不放宽权限 |
| AI-12 | 隐私边界 | 缺 | P1 / S | 明确「什么会被发送」；默认不发；可一键清空 | 设置页可见 |
| AI-13 | 事件日志 | 缺 | P2 / S | 本地记录 agent 状态变化，便于复盘 | 可关闭；限大小 |
| AI-14 | 等待提示音 | 缺 | P2 / S | blocked 可选提示音/语音；勿扰时静音 | 与 STAT-03 联动 |
| AI-15 | 任务完成总结 | 缺 | P3 / M | 会话结束时生成摘要（显式） | 与 AI-06 共用 |

### 2.16 A11Y：无障碍、国际化、CJK

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| A11Y-01 | 屏幕阅读器 | 缺 | P2 / L | 暴露终端文本、当前行、通知、控件；可读选区 | AccessKit；终端文本要按行聚合 |
| A11Y-02 | 高对比度 | 缺 | P2 / M | 高对比主题、焦点环、状态不只靠颜色 | 与 UI-24 共用 |
| A11Y-03 | 全键盘导航 | 部分 | P1 / M | 不用鼠标完成：标签、窗格、面板、设置、对话框 | 焦点顺序与快捷键表 |
| A11Y-04 | 焦点可见 | 部分 | P1 / S | 键盘焦点始终有明显边框/高亮 | 不要只靠细微颜色差 |
| A11Y-05 | 减少动态 | 缺 | P1 / S | 遵循系统设置；宠物/面板/光标动画可关 | 读平台偏好 |
| A11Y-06 | 字体缩放 | 部分 | P1 / S | UI 字号与终端字号独立缩放；快捷键 | 现有终端缩放；补 UI 缩放 |
| A11Y-07 | 不只靠颜色 | 缺 | P2 / S | 失败/成功同时用图标/文字/形状 | 标签状态点加形状 |
| A11Y-08 | 通知播报 | 缺 | P2 / M | 屏幕阅读器播报通知与命令完成 | 与 A11Y-01 |
| A11Y-09 | 命令完成播报 | 缺 | P2 / S | 「命令完成，退出码 0，用时 2 秒」 | 可配置详细程度 |
| A11Y-10 | IME 候选窗 | 部分 | P0 / M | 候选窗贴光标；不遮挡输入行；多屏正确 | 与 INP-04 |
| A11Y-11 | CJK 宽度/回退 | 部分 | P1 / M | 中文/日文/韩文字体回退；模糊宽度可选；标点正确 | 与 PROTO 字素/宽度 |
| A11Y-12 | 本地化 | 部分 | P1 / M | 中/英完整；错误信息、设置搜索、空状态都翻译 | 现有 `language`；补遗漏与复数 |
| A11Y-13 | RTL 声明 | 缺 | P3 / S | 明确「不支持 BiDi 重排」；至少不把 RTL 文本显示成乱序控制符 | 文档 + 安全清洗 |
| A11Y-14 | 触屏 | 缺 | P3 / M | 触屏滚动、长按选择、双击词 | 平台事件 |
| A11Y-15 | 放大镜兼容 | 缺 | P3 / S | 系统放大镜/缩放时文字不糊 | 高 DPI 渲染 |
| A11Y-16 | 光标对比度 | 缺 | P2 / S | 光标在浅色/深色背景都可见；可加描边 | 主题计算 |

### 2.17 PERF：性能、诊断、测试

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| PERF-01 | 行级 damage | 缺 | P0 / M | 只重绘脏行/脏区；滚动复用 | 当前 `render` 每帧重建整屏 `Frame`（`view.rs:352`），是首要成本 |
| PERF-02 | Shaping 缓存 | 缺 | P0 / M | 相同文本/样式复用 shaping；字体/缩放变化才失效 | 以 `(text, attrs, font)` 为键；限制缓存大小 |
| PERF-03 | 帧调度 | 部分 | P1 / M | vsync、合帧、空闲不跑动画帧；高刷 | 光标闪烁只失效光标区域 |
| PERF-04 | 输入延迟测量 | 缺 | P1 / M | 记录 keydown→present；p50/p99；可导出 | 时间戳 + present 回调 |
| PERF-05 | 启动 trace | 缺 | P1 / S | 冷启动各阶段耗时；P95 < 300ms | 延迟 Git/Files/主题字体/更新检查 |
| PERF-06 | 滚动缓冲压缩 | 缺 | P1 / L | 默认 50k，100k 可选；内存可控 | 不能只改数字：100k×200 列约 0.5GB；样式表压缩/按需 |
| PERF-07 | 后台搜索 | 部分 | P1 / M | 搜索不阻塞 UI；可取消；每行纯文本镜像 | 100k 行首命中 < 50ms |
| PERF-08 | 大粘贴 | 部分 | P1 / M | 分块写入 PTY；进度/取消；不冻结 | 背压与限速 |
| PERF-09 | 每会话内存 | 缺 | P2 / M | 隐藏标签停止重绘；base RSS < 80MB，隐藏会话 < 10MB | 目标值写进预算 |
| PERF-10 | 性能面板 | 缺 | P1 / M | FPS、frame p50/p99、丢帧、PTY 吞吐、parse/shape/paint、脏行、RSS | 开发与用户都能开；默认隐藏 |
| PERF-11 | 慢帧日志 | 缺 | P2 / S | 超过阈值记录原因（大输出/字体/动画） | 环形缓冲，避免日志爆炸 |
| PERF-12 | PTY 吞吐/背压 | 部分 | P2 / M | 大量输出不丢、不卡 UI；可限制每帧处理字节 | 读取线程与渲染解耦 |
| PERF-13 | 泄漏检测 | 缺 | P2 / M | 长跑会话内存曲线；图片/字体/会话缓存上限 | 开发构建统计 |
| PERF-14 | 崩溃恢复 | 缺 | P2 / M | 每 30s 原子快照；启动检测非正常退出并提示 | 与 TAB/会话恢复共用状态 |
| PERF-15 | 基准测试 | 缺 | P2 / M | criterion：解析、搜索、resize、shape、启动 | CI 跑阈值 |
| PERF-16 | vttest / esctest | 缺 | P1 / M | 协议回归；覆盖光标、滚动、模式、颜色 | 自动化脚本；输出可对比 |
| PERF-17 | Fuzz | 缺 | P2 / M | 随机字节喂 VT/OSC 解析器；不 panic、不越界 | cargo-fuzz；限内存 |
| PERF-18 | 截图回归 | 缺 | P2 / L | 固定输入 → 固定帧 → 像素对比 | 需要稳定字体/平台；允许阈值 |
| PERF-19 | 跨平台 CI | 部分 | P1 / M | Windows/macOS/Linux 构建 + 测试；Wayland/X11 | 现有 CI 补矩阵 |
| PERF-20 | 日志分级/诊断包 | 部分 | P1 / S | `RUST_LOG` 分级；一键导出诊断（版本、配置、最近日志、OSC 统计） | 脱敏 |
| PERF-21 | Debug overlay | 缺 | P2 / S | damage 热图、网格线、字形 atlas、帧耗时 | 开发快捷键 |
| PERF-22 | 协议检查器 | 缺 | P2 / M | 显示最近 OSC/CSI/APC/DCS 序列、来源、是否被处理 | 用于「为什么没渲染」 |
| PERF-23 | 会话录制/回放 | 缺 | P3 / L | 记录 PTY 字节 + 时间戳，回放复现 bug | asciinema 格式；隐私提示 |
| PERF-24 | 性能预算 CI | 缺 | P3 / M | 超过预算让 CI 失败（启动、搜索、帧） | 与 PERF-15 共用 |

### 2.18 PLAT：平台集成、CLI、发布

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| PLAT-01 | Windows 跳转列表 | 缺 | P3 / S | 任务栏右键：最近目录、新建标签 | `ICustomDestinationList` |
| PLAT-02 | 任务栏进度/通知 | 缺 | P1 / M | 进度、错误、overlay 图标；点击激活会话 | `ITaskbarList3`；与 STAT-04 共用 |
| PLAT-03 | “在此打开” | 缺 | P2 / M | 资源管理器右键「在 ELYTerminal 中打开」；安装/卸载注册表项要显式 | 不劫持默认终端 |
| PLAT-04 | 默认终端关联 | 缺 | P3 / XL | Windows 默认终端涉及系统设置与签名，个人 fork 不做 | 明确不做，避免承诺 |
| PLAT-05 | macOS 菜单/服务 | 部分 | P1 / S | 菜单栏命令、Services、Dock 菜单 | 现有菜单；补 Services |
| PLAT-06 | macOS 安全输入 | 缺 | P2 / M | 密码场景启用 secure event input | 见 INP-13 |
| PLAT-07 | macOS 标签页/窗口 | 缺 | P3 / M | 原生 window tabbing、全屏、舞台管理器 | 与 TAB-22 协调 |
| PLAT-08 | Linux 桌面项 | 已有 | — | `.desktop`、图标 | 保持 |
| PLAT-09 | Wayland layer-shell 快速终端 | 缺 | P2 / L | 下拉终端不用抢焦点；多显示器 | 与 TAB-23 |
| PLAT-10 | Portal / IME | 部分 | P1 / M | 文件选择、截图、输入法在 Wayland 下正常 | xdg-desktop-portal |
| PLAT-11 | CLI | 缺 | P0 / M | `-d/--working-directory`、`-e/--command`、`--profile`、`--tab`、`--title`、`--hold` | 退出码透传；与单实例协作 |
| PLAT-12 | 单实例 IPC | 缺 | P1 / M | 第二次启动把请求交给已运行实例；命名管道/Unix socket | 参数校验；不要把任意命令当可信 |
| PLAT-13 | URL scheme | 缺 | P2 / M | `elyterminal://new?cwd=...&profile=...`；命令参数默认不执行 | 白名单；危险参数需确认 |
| PLAT-14 | 文件管理器集成 | 缺 | P2 / S | 拖放打开目录、`Open with`、最近文件 | 平台注册 |
| PLAT-15 | 更新检查 | 已有 | — | 查 GitHub release | 保持；补跳过版本/通道 |
| PLAT-16 | 崩溃包 | 缺 | P2 / M | panic hook + backtrace + 最近日志 + 配置快照 | 本地导出，自动上传关闭 |
| PLAT-17 | 日志查看器 | 缺 | P2 / S | 设置里查看/过滤/复制日志；打开日志目录 | 脱敏 |
| PLAT-18 | 便携模式 | 缺 | P2 / S | exe 同目录 `portable.json`/`data/`；不写用户目录 | 优先级高于安装版 |
| PLAT-19 | 配置目录规范 | 部分 | P1 / S | `%APPDATA%`、`~/Library/Application Support`、`$XDG_CONFIG_HOME` | 文档写清楚；迁移旧路径 |
| PLAT-20 | 发布通道 | 缺 | P3 / S | stable/nightly；版本号与 commit 可见 | 个人 fork 可只做 nightly |
| PLAT-21 | Shell 补全 / man page | 缺 | P3 / S | `elyterminal <TAB>`；`man elyterminal` | 生成脚本 |
| PLAT-22 | 文档/示例配置 | 部分 | P1 / S | README 补 CLI、配置、快捷键；`examples/` 放配置 | 与 `--help` 一致 |
| PLAT-23 | 插件 API | 缺 | P3 / XL | 先不做运行时；用 CLI/IPC + 配置覆盖 80% 场景 | 明确边界，避免过早设计 |

### 2.19 PROTO：协议与仿真

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| PROTO-01 | DA1 诚实作答 | 缺 | P0 / S | 现在 alacritty 回 `CSI ? 6 c`（VT102），程序会降级；按真实能力作答 | 在 tap 拦截 `CSI c`；未实现 sixel 不要报 `4` |
| PROTO-02 | DA2 / XTVERSION | 部分 | P0 / S | DA2 已有；补 `CSI > 0 q` → `DCS > \| ELYTerminal(版本) ST` | 身份字符串可配置 |
| PROTO-03 | DECRQM | 部分 | P0 / S | 9001/1015/1016/2026/2027/2048 如实回答；未知答 0 | 现有实现未知一律 0、2026 恒 Reset |
| PROTO-04 | XTGETTCAP | 缺 | P1 / M | `TN/Co/RGB/Tc/Ms/Su`；tmux 会代答 | 只答真实支持项 |
| PROTO-05 | DECRQSS | 缺 | P2 / S | SGR、DECSTBM、DECSCUSR 等查询 | 只报告必要状态 |
| PROTO-06 | 同步输出 2026 | 部分 | P1 / S | 透传已有；honest DECRQM；1s 超时保护 | 注意 tmux 透传 |
| PROTO-07 | 字素簇 2027 | 缺 | P1 / L | UAX #29；ZWJ、旗帜、肤色、VS16、组合符 | fork/扩展 alacritty 网格；`unicode-segmentation` |
| PROTO-08 | 带内 resize 2048 | 缺 | P2 / M | `CSI ? 2048 h` + `CSI 48;rows;cols;hpx;wpx t` | 启用后立即发一次当前尺寸 |
| PROTO-09 | 配色变更通知 2031 | 缺 | P3 / S | 主题变化时通知程序 | 只在程序请求时 |
| PROTO-10 | 47/1047/1048 | 缺 | P1 / S | 补旧备用屏与保存光标兼容 | vte 目前只识别 1049 |
| PROTO-11 | modifyOtherKeys | 缺 | P1 / M | 见 INP-02 | vte handler 为空 |
| PROTO-12 | 鼠标 1015/1016 | 缺 | P1 / S | urxvt 编码、像素坐标；滚轮 64–67 | 与 1006 互斥 |
| PROTO-13 | OSC 133 扩展 | 部分 | P1 / S | `k=i/r/c/s`、`aid`、`cl`、`err`；633 兼容 | 现有 A/N/B/I/C/D |
| PROTO-14 | OSC 8 参数 | 部分 | P1 / S | `id=` 已有；补参数透传、scheme 白名单、超长拒绝 | 打开前显示真实目标 |
| PROTO-15 | OSC 4/10/11/12/104 | 部分 | P1 / S | 动态配色与查询；按 pane 隔离 | alacritty 提供；确认 UI 一致性 |
| PROTO-16 | OSC 52 策略 | 部分 | P0 / S | 写可用、读默认关、限长、确认 | 见 SEL-08 |
| PROTO-17 | OSC 1337 / 22 / 50 | 缺 | P2 / M | iTerm2 图片/徽标/变量；光标形状；字体查询 | OSC 50 字体加载建议忽略 |
| PROTO-18 | kitty graphics | 缺 | P2 / XL | APC、transmit/display/delete/query、placement、z-index | 先 PNG + `t=d`；配额；禁远程文件读取 |
| PROTO-19 | Sixel | 缺 | P3 / L | DCS、调色板、RLE、DECSDM | 解码炸弹防护 |
| PROTO-20 | iTerm2 inline images | 缺 | P3 / M | `OSC 1337;File=inline=1` | 实现最简单；base64 膨胀 |
| PROTO-21 | tmux passthrough | 部分 | P1 / M | `DCS tmux;` 双写 ESC；2026/OSC 8/52 组合 | 测试嵌套 mux；文档化配置 |
| PROTO-22 | 标题栈/清洗 | 部分 | P1 / S | C0/C1/BiDi 清理；限长 | 防止标题注入 |
| PROTO-23 | 宽度/模糊宽度 | 部分 | P2 / M | UAX #11；A 宽度按 locale/设置 | 与 A11Y-11 |
| PROTO-24 | BiDi | 缺 | P3 / XL | 明确不支持；至少不破坏逻辑顺序 | 不建议半套实现 |

### 2.20 SEC：安全与隐私

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| SEC-01 | 粘贴保护 | 部分 | P0 / S | 多行/控制字符/超长先预览；不自动执行结尾换行 | 现有括号粘贴；补确认 |
| SEC-02 | 危险命令提示 | 缺 | P2 / M | `sudo`、`rm -rf`、`curl \| sh` 等高亮/确认（可配） | 只提示，不阻止；不要误报太多 |
| SEC-03 | OSC 52 | 部分 | P0 / S | 写限长、大内容确认、读默认拒绝 | 见 SEL-08 |
| SEC-04 | OSC 8 白名单 | 缺 | P0 / S | 只允许 http/https/mailto；拒绝 `javascript:`、`data:`、控制字符 | 显示真实目标 |
| SEC-05 | 标题清洗 | 缺 | P0 / S | 去 C0/C1、U+202A–202E、U+2066–2069；限长 | 不当 markup |
| SEC-06 | 通知/进度限流 | 部分 | P1 / S | 限流、去重、限长；`msg` 拒绝控制字符 | 现有部分限流 |
| SEC-07 | 图片限制 | 缺 | P2 / M | 尺寸/像素/解压比/内存配额；解码在后台 | 未实现图形前也要定规则 |
| SEC-08 | 本地文件读取 | 缺 | P2 / M | kitty `t=f/t=s` 在远程会话默认禁用 | 只允许 direct data |
| SEC-09 | 不可信会话模式 | 缺 | P2 / M | 一键关闭 OSC 52/通知/链接/文件传输/图片 | 用于生产/陌生主机 |
| SEC-10 | known_hosts 严格 | 缺 | P1 / S | 主机密钥变化必须显式确认；显示指纹 | 与 CONN-06 |
| SEC-11 | 凭据存储 | 缺 | P1 / M | 系统凭据库；不写配置文件明文 | 与 CONN-20 |
| SEC-12 | 日志脱敏 | 缺 | P1 / S | 路径、token、命令参数可配置掩码；诊断包默认脱敏 | 导出前预览 |
| SEC-13 | URL scheme 校验 | 缺 | P2 / S | 参数白名单；命令参数默认不执行 | 与 PLAT-13 |
| SEC-14 | 配置权限 | 缺 | P2 / S | Windows/macOS/Linux 下配置文件不可被其他用户读 | 0600 / ACL |
| SEC-15 | 更新校验 | 缺 | P3 / S | 校验哈希/签名；未签名构建明确提示 | 不做自动替换 |
| SEC-16 | 零遥测 | 已有 | — | 不收集使用数据 | 保持；文档声明 |
| SEC-17 | 安全键盘输入 | 缺 | P2 / M | 见 INP-13 | 平台能力 |
| SEC-18 | 剪贴板读取提示 | 部分 | P1 / S | 读取已默认关；若开放，逐次确认 | 见 SEL-08 |
| SEC-19 | 子进程权限 | 部分 | P2 / M | 不以更高权限运行子进程；提权显式 | 见 SHELL-11 |
| SEC-20 | 沙箱 | 缺 | P3 / XL | 不可信命令的受限环境 | 个人 fork 不做；用系统沙箱/容器 |

### 2.21 FUN：Ely 与趣味功能

| ID | 功能 | 状态 | 优先级/成本 | 行为与细节 | 实现要点 |
|---|---|---|---|---|---|
| FUN-01 | Ely 情绪联动 | 部分 | P2 / S | 构建成功开心、失败沮丧、agent blocked 着急、空闲睡觉 | 读 STAT 状态；不要影响终端 |
| FUN-02 | Ely 走动 | 部分 | P2 / S | 在窗格边缘走动、探头、趴下；可关 | 现有 rig/moods；限制频率 |
| FUN-03 | 宠物互动 | 缺 | P3 / S | 点击/拖拽有反应；双击打开设置/关于 | 不遮挡输入区 |
| FUN-04 | 宠物主题 | 缺 | P3 / S | 随主题换色/配饰；节日皮肤 | 资源可关 |
| FUN-05 | 本地统计 | 缺 | P3 / S | 命令数、失败数、最常用目录、活跃时段；只存本地 | 无上传；可清除 |
| FUN-06 | 氛围效果 | 缺 | P3 / S | 极轻的呼吸/光标光晕；减少动态时关闭 | 不能影响可读性 |
| FUN-07 | 彩蛋 | 缺 | P3 / S | 输入 `ely` 有反应；关于页小交互 | 不要干扰正常命令 |
| FUN-08 | 节日主题 | 缺 | P3 / S | 自动或手动切换节日配色/宠物 | 可关 |
| FUN-09 | 截图模式 | 缺 | P3 / S | 隐藏 UI/宠物，只留终端；导出 PNG | 注意剪贴板/隐私 |
| FUN-10 | 会话故事/回放 | 缺 | P3 / L | 把一次会话的命令与输出做成可回放的时间线 | 与 PERF-23 共用；隐私 |
| FUN-11 | 桌面宠物窗口 | 缺 | P3 / M | Ely 独立小窗，能拖到桌面；与主窗互动 | 平台窗口；可关 |
| FUN-12 | 打字音效 | 缺 | P3 / S | 可选机械键盘音；默认关 | 延迟与性能；勿扰时静音 |

---

## 3. 重点功能规格卡

### 3.1 SHELL-01 / SHELL-04 / SHELL-05：Shell 目录与 WSL/Git Bash

**目标**：用户打开设置，能看到这台机器上真实存在的 shell，并一键建 profile。

**交互**

- 设置 → 终端 → Shell：不再是单个路径输入框，而是「已发现」列表 + 「自定义路径」。
- 每行：图标、名称（`Git Bash` / `WSL: archlinux` / `Windows PowerShell 5.1`）、路径、来源标签、版本、测试按钮。
- 「刷新」重新扫描；「添加自定义」允许填任意可执行文件与参数。
- 新标签按钮旁有下拉，可选 profile。

**数据**

```text
ShellEntry { id, kind, name, path, args, source, version? }
kind = Local | GitBash | Wsl { distro, default } | Msys | Cygwin | Nu | ...
```

**实现**

- 后台扫描，结果进 `Global`；首帧不等待。
- 注册表优先，PATH 次之，已知路径兜底。
- `System32\bash.exe` 标记为 WSL 启动器，不标成 Git Bash。
- `wsl -l -v` 输出按 UTF-16LE 解码；只取首列名字、末列版本。

**边界**

- 没装任何非默认 shell：列表只显示 PowerShell/cmd，不显示空分组。
- 路径含空格/中文：启动参数不加引号（`Command`/`Shell::new` 自己处理），显示时正常。
- WSL 未启动：显示「Stopped」，启动时给 1–2 秒的「正在启动发行版」提示。

**验收**

- [ ] 列表准确列出 PowerShell、cmd、Git Bash、archlinux、Ubuntu-22.04。
- [ ] 点 Git Bash 能在指定 cwd 打开，`pwd` 正确。
- [ ] 点 WSL 能在 `/mnt/c/...` 打开，左侧 badge 显示发行版。
- [ ] 卸载 Git 后刷新，列表不再出现 Git Bash。

### 3.2 PATH-01 / PATH-03 / PATH-07：Location 与路径映射

**目标**：无论会话在 Windows、WSL 还是 SSH，cwd、拖放、面板、复制路径都用同一套语义。

**交互**

- 窗格顶部面包屑：`WSL: archlinux › home › user › src`。
- 拖文件进 WSL：插入 `/mnt/c/...`；拖进 Git Bash：插入 `/c/...`；拖进 pwsh：插入 `C:\...` 并正确引用。
- 右键路径：复制为 Windows / WSL / POSIX / 相对 / URI。

**实现**

```rust
enum Location {
    Local(PathBuf),
    Wsl { distro: String, path: String },
    Ssh { profile: String, path: String },
    Container { .. },
}
```

- `from_osc7(url, session)` 按会话的 distro 映射，不按 URL host 猜。
- `wslpath` 异步 + 缓存；失败回退到字符串规则。
- Git Bash 的 `/c/`、`/usr/` 分开处理。
- 拖放走括号粘贴。

**边界**

- UNC 路径：cmd 用 `pushd`，其他 shell 正常。
- 远程 cwd：不调用本地 `metadata`；「打开文件夹」走 SFTP 或提示。
- 路径不存在：回退顺序明确，不要静默落到 `C:\`。

**验收**

- [ ] WSL 里 `cd /tmp` 后，左栏/Files 面板显示 `\\wsl.localhost\archlinux\tmp`。
- [ ] Git Bash 里 `cd /c/Users` 后，面包屑显示 `C:\Users`。
- [ ] 拖 `C:\a b\c.txt` 进 WSL 得到 `/mnt/c/a\ b/c.txt`。
- [ ] SSH 会话的「在资源管理器中显示」被禁用并解释原因。

### 3.3 TAB-15 / TAB-16 / TAB-03：标签与分屏的人体工学

**目标**：现有骨架从「能分」变成「愿意天天用」。

**交互**

- 拖分隔条改比例；双击均分；比例在会话恢复后保留。
- `Ctrl+Shift+Enter`（示例）放大当前窗格，再按还原；状态栏提示「已放大」。
- 左栏标签上下拖动排序，插入线清晰；`Alt+Up/Down` 也能移动。
- 关闭有前台进程的标签/窗格时确认；「全部关闭」汇总数量。
- 非焦点窗格边缘压暗；焦点窗格有 1px 高亮。

**实现**

- `Pane.width/height` 从十分位改成 `f32` 比例或保留整数但支持任意值；布局渲染按比例分配。
- 放大只改绘制：`zoomed: Option<usize>`，不修改 `Tab.panes`。
- 拖拽排序用 GPUI 的 drag/drop；持久化顺序。
- 关闭确认检查 PTY 前台进程组（Unix）或子进程（Windows）。

**验收**

- [ ] 拖分隔条后比例保持，重启后仍保持。
- [ ] 放大窗格时其他窗格不消失，退出后比例不变。
- [ ] 关闭运行中 `cargo watch` 的标签会确认。
- [ ] 10 个标签拖动排序不闪烁、不丢焦点。

### 3.4 CFG-01 / CFG-02 / CFG-09：配置、热重载与键位

**目标**：设置窗口和 `config.toml` 是同一模型的两个视图，改哪边都生效。

**交互**

- 首次运行生成带注释的 `config.toml`；设置窗口每个控件旁有「在文件中编辑」。
- 保存文件后 200ms 内生效；错误显示文件、行、列，旧配置继续用。
- 键位页：搜索 action、录制新键、冲突提示、恢复默认、按平台查看。

**实现**

- `toml_edit` 保留注释；读取 → 校验 → 构建 `Settings` → 原子替换。
- 文件监听用 mtime + hash，避免编辑器「保存两次」抖动。
- 冲突检测：同一 context 下同一组合键映射到两个 action 时阻止并提示。
- 设置窗口写入时加「自己写的」标记，避免监听回环。

**验收**

- [ ] 改 `font_size` 保存，窗口字体立即变化，无需重启。
- [ ] 写错类型，弹出可点击的错误，界面仍可用。
- [ ] 把「新建标签」改成 `Ctrl+Alt+T`，冲突时提示，改完立即生效。
- [ ] 设置窗口改主题后，文件里对应行更新且注释保留。

### 3.5 TAB-22 / TAB-23 / PLAT-12：多窗口、快速终端与单实例

**目标**：终端像系统工具，而不是一个必须切换过去的应用。

**交互**

- `Ctrl+Shift+N` 新建窗口；标签可拖出/合并。
- 全局热键（默认 `Win+\`` / `Cmd+\``）呼出快速终端；失焦隐藏；`Esc` 隐藏。
- 从资源管理器/命令行再次启动，默认在当前窗口开新标签（可配置为开新窗口）。

**实现**

- 多窗口共享 `Settings`、`ShellCatalog`、主题；会话各自持有 PTY。
- 快速终端是独立 window，置顶/无边框；Wayland 用 layer-shell，macOS 用 NSPanel，Windows 用 HWND + 置顶。
- 单实例：Windows 命名管道，Unix domain socket；消息只接受结构化请求（cwd、profile、命令需确认）。

**验收**

- [ ] 两个窗口各自切换标签互不影响。
- [ ] 任意应用按全局热键，300ms 内出现快速终端。
- [ ] 第二次 `elyterminal -d .` 在当前窗口开新标签，cwd 正确。
- [ ] 快速终端失焦后隐藏，不抢焦点。

### 3.6 BLOCK-01 / BLOCK-05 / BLOCK-08：命令块

**目标**：把「一条命令」变成可选、可折叠、可重跑、可诊断的对象。

**交互**

- 每条命令左侧细条：运行中蓝色、成功绿色、失败红色、中断灰色。
- hover 显示命令、时长、退出码；点击细条选中整块。
- `Ctrl+Up/Down` 跳块；`Ctrl+Shift+Up/Down` 只跳失败块。
- 右键：复制命令、复制输出、重跑、折叠、解释错误、导出。
- 折叠后只留命令行与状态，输出区显示「已折叠 N 行」。

**实现**

- `Block { prompt_range, command_range, output_range, exit, started, ended }`，由 OSC 133 事件构建。
- 块是逻辑坐标，滚动/重排后仍能定位；被滚动缓冲淘汰时标记失效。
- 折叠只影响绘制，不改变网格内容。
- 重跑把命令写到 PTY，默认不自动回车，需确认。

**验收**

- [ ] `cargo test` 成功/失败显示不同颜色与退出码。
- [ ] 折叠 1000 行输出后滚动流畅，展开恢复。
- [ ] 重跑不会误执行上一条命令；确认框显示命令全文。
- [ ] 没有 shell 集成时块功能隐藏，不显示空 gutter。

### 3.7 STAT-07 / STAT-08 / AI-02：Agent 状态与等待队列

**目标**：多开 agent 时，用户一眼知道「哪个在跑、哪个在等我」。

**交互**

- 左栏每个会话行显示状态点 + 文案（运行中 / 等待输入 / 完成 / 失败）。
- 顶部「等待」按钮：列出所有 blocked 会话，按等待时长排序。
- blocked 超过 5 秒：任务栏角标 + 一条聚合通知；点击跳到该会话。
- 勿扰模式下只角标不弹窗。

**实现**

- 统一状态机：`Idle | Working | Blocked | Error | Done`，来源 OSC 7501、OSC 133 D、子进程退出、焦点。
- 去重：同一会话 30 秒内只通知一次；blocked 升级一次。
- 任务栏 API 见 PLAT-02。

**验收**

- [ ] agent 等待审批时 5 秒内出现角标与通知。
- [ ] 点击通知定位到正确窗格与命令块。
- [ ] 10 个 agent 同时 blocked，只出现一条聚合通知。
- [ ] 勿扰模式无弹窗，但角标仍在。

### 3.8 PERF-01 / PERF-02 / PERF-10：渲染性能

**目标**：4K 全屏、快速滚动、多会话下不卡。

**交互**

- 开发模式 `Ctrl+Shift+F12` 打开性能面板：FPS、frame p50/p99、丢帧、脏行、PTY 吞吐、RSS。
- 慢帧超过 16ms 记录原因（大输出/字体/动画/图片）。

**实现**

- 行级 damage：`Term` 的 damage 已是行/区，渲染只遍历脏行；缓存每行 shaping。
- shaping 缓存键 `(text, attrs, font_id, scale)`；超过上限 LRU。
- 帧合并：PTY 输出在一帧内聚合成一次绘制；空闲不请求动画帧。
- 光标闪烁只失效光标矩形，不整屏。

**验收**

- [ ] 4K 全屏 `yes` 滚动 p99 < 8ms（目标值，按机器调整）。
- [ ] 输入回显 ≤ 1 帧。
- [ ] 20 个隐藏会话时 RSS 增量 < 200MB。
- [ ] 性能面板能指出一次卡顿的具体阶段。

### 3.9 SEC-01 / SEC-03 / SEC-04：安全默认值

**目标**：默认配置下，恶意输出不能读剪贴板、不能静默打开危险链接、不能伪造标题。

**交互**

- 粘贴多行 → 预览 + 「粘贴 12 行」确认；含 `ESC`/控制字符时高亮。
- OSC 52 写入超过阈值（如 100KB）→ 确认；读取始终拒绝。
- 链接只允许 http/https/mailto；`file:` 交给系统但显示路径。

**实现**

- 粘贴管线统一剥 ESC/C0，再按 bracketed paste 包装。
- OSC 52 长度在 tap 层限制；读请求直接丢弃并记日志。
- 标题清洗函数统一处理 C0/C1/BiDi 控制符与长度。

**验收**

- [ ] `printf '\e]52;c;?\a'` 不能读到剪贴板。
- [ ] `\e]8;;javascript:alert(1)\a` 不可点击。
- [ ] 标题含 `\e[31m` 不改变窗口标题颜色/内容。
- [ ] 粘贴 100 行不会直接执行。

### 3.10 PROTO-01 / PROTO-03 / PROTO-04：能力协商

**目标**：程序问「你能做什么」时，回答真实能力，避免降级或崩溃。

**交互**：无 UI，纯协议；诊断页可查看最近的能力查询与回复。

**实现**

- tap 拦截 `CSI c`、`CSI > c`、`CSI > 0 q`、`CSI ? Ps $ p`、`DCS + q`。
- DA1 回复按功能开关拼装；sixel/kitty graphics 未实现时不报。
- DECRQM 用真实模式表；未知模式答 0。
- XTVERSION 返回 `ELYTerminal(0.1.7)`。

**验收**

- [ ] `vttest` 的设备属性页通过。
- [ ] `infocmp`/`tmux` 能识别终端能力。
- [ ] 未实现 sixel 时，程序不会尝试发送 sixel。
- [ ] 同步输出期间 DECRQM 2026 的回答与实际状态一致。

---

## 4. 路线图

### 4.1 P0：两周内（v0.2 连接）

| ID | 事项 | 完成标准 |
|---|---|---|
| SHELL-01/03/04/05/07/12/13/14 | Shell 目录、默认 shell、Git Bash、WSL、cwd、失败诊断、集成修复 | 本机五类 shell 全部可见可启动 |
| CFG-01/02/03/04/09 | TOML、热重载、双向同步、迁移、键位 | 改文件即生效；改键不冲突 |
| PATH-01/02/03/04/05/07 | Location、OSC 7/9;9、WSL/Git Bash 映射、回退、拖放 | 四个路径验收全过 |
| CONN-01/02/03 | SSH config 导入、`ssh -G`、主机列表 | 列表来自真实 config，一键连接 |
| TAB-01/05/10/12/15/16 | 树形标签、重命名、角标、关闭确认、比例、放大 | 日常标签/分屏操作不缺 |
| PERF-01/02/10 | 行级 damage、shaping 缓存、性能面板 | 4K 滚动不掉帧，可观测 |
| SEC-01/03/04/05 | 粘贴、OSC 52、链接白名单、标题清洗 | 安全验收全过 |
| PROTO-01/03/16 | DA1、DECRQM、OSC 52 | 能力诚实，读剪贴板被拒 |

### 4.2 P1：一到两个月（v0.3 工作台 + v0.4 正确性）

| ID | 事项 | 完成标准 |
|---|---|---|
| TAB-22/23/26 | 多窗口、快速终端、几何记忆 | 全局热键 300ms 呼出 |
| CFG-08/12/14 + PLAT-11 | profile 继承、设置搜索、错误定位、CLI | `elyterminal -d .` 可用 |
| BLOCK-01..08/10/11 | 命令块、折叠、重跑、导航、状态、导出、历史 | 失败命令一键定位/重跑 |
| STAT-02/03/04/08/09/10/11 | 聚合、勿扰、任务栏、agent、等待、长任务、跳转 | blocked 5 秒内可见 |
| FIND-02/03/05/06/08/12 | 搜索选项、选区、计数、滚动条标记、跳转、性能 | 100k 行 < 50ms |
| INP-04/09/10/14 + PROTO-10/12/13 | IME、粘贴、鼠标、速查、47/1047、1015/1016、133 扩展 | 中文输入与鼠标兼容 |
| PROTO-02/04/06/07/11/21/22 | XTVERSION、XTGETTCAP、2026、字素、modifyOtherKeys、tmux、标题 | vttest/esctest 通过 |
| SEL-02/03/07/12 | 智能选词、动作菜单、粘贴预览、复制路径 | 选区可操作 |
| CONN-05/06/07/10/18/20/21 | 认证、known_hosts、跳板、重连、徽标、凭据、TERM | SSH 日常可用 |

### 4.3 P2：一个季度（v0.5 远程 + v0.6 Agent + v0.7 打磨）

| ID | 事项 | 完成标准 |
|---|---|---|
| CONN-08/12/13/16 | 端口转发、远程 tmux、SFTP、容器 | 远程工作流闭环 |
| XFER-01..06/09/10 | SFTP 浏览、传输、远程编辑、队列、映射、安全 | 不离开终端改远程文件 |
| TAB-09/17/19/20/21/28/30 | 工作区、窗格移动、广播、同步滚动、布局、拖放分屏、折叠 | 多任务布局可保存 |
| AUTO-01..07 | 启动命令、snippets、触发器、高亮、收藏 | 常用流程可复用 |
| AI-03/05/07/08/11/12 | 看板、解释、建议、diff、CLI、隐私 | agent 状态可聚合 |
| UI-02/05/06/09/13/15/24/26 | 主题导入、profile 主题、背景、可变字体、下划线、调色板、高对比、滚动条 | 外观可迁移/可定制 |
| A11Y-01/02/03/06/11/12 | 屏幕阅读器、高对比、键盘、缩放、CJK、i18n | 全键盘可完成日常操作 |
| PLAT-03/09/10/13/16/17/18 | “在此打开”、layer-shell、portal、URL、崩溃包、日志、便携 | 平台集成可用 |
| PERF-06/07/12/14/19/20 | 缓冲、搜索、背压、崩溃恢复、CI、诊断 | 长跑稳定 |
| PROTO-08/17/18 | 2048、OSC 1337/22、kitty graphics | 图片协议第一版 |

### 4.4 P3：实验或以后

- TAB-02 顶部标签条、TAB-06 预览、TAB-07 总览、TAB-24 画中画。
- XFER-04 Zmodem、XFER-07 断点续传。
- AUTO-08..13 触发器执行、宏录制、每目录脚本、钩子。
- AI-04/06/15 成本、摘要、任务总结。
- FUN-05..12 统计、回放、桌面宠物、音效。
- PERF-18/23/24 截图回归、录制回放、性能预算 CI。
- PLAT-20/21/23 发布通道、补全、插件 API。
- PROTO-19/20 Sixel、iTerm2 图片；PROTO-24 BiDi（或明确不做）。

---

## 5. 配置与数据模型草案

### 5.1 `config.toml`（草案）

```toml
version = 1

[general]
language = "zh-CN"
start_in = "session"          # home | session | fixed | last
default_profile = "pwsh"

[appearance]
theme = "ely-dark"
dark_theme = "ely-dark"
font_family = "JetBrains Mono"
font_size = 13.0
line_height = 1.2
ligatures = true
padding = [10, 8]
opacity = 0.88
blur = true

[terminal]
scrollback = 50_000
cursor_style = "block"        # block | bar | underline | hollow
cursor_blink = true
copy_on_select = false
osc52 = "write"               # off | write | ask
notifications = "aggregate"   # off | all | aggregate
confirm_multiline_paste = true

[security]
allow_osc8 = ["http", "https", "mailto"]
title_sanitize = true
untrusted_mode = false

[keys]
new_tab = "ctrl-shift-t"
split_right = "ctrl-shift-d"
split_down = "ctrl-shift-alt-d"
command_palette = "ctrl-shift-p"
quick_terminal = "win-`"

[[profiles]]
id = "pwsh"
name = "PowerShell 5.1"
kind = "local"
program = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
args = ["-NoLogo", "-NoExit"]
cwd = "home"
color = "blue"

[[profiles]]
id = "wsl-arch"
name = "WSL: archlinux"
kind = "wsl"
distro = "archlinux"
cwd = "session"
color = "green"

[[profiles]]
id = "git-bash"
name = "Git Bash"
kind = "git-bash"
program = "C:\\Program Files\\Git\\bin\\bash.exe"
cwd = "session"
color = "orange"
```

> 说明：字段名是草案，最终以 `PRD.md` 为准；`kind` 决定启动配方与路径转换，不靠文件名猜。

### 5.2 Profile 字段

| 字段 | 说明 |
|---|---|
| `id` | 稳定标识，session restore 引用它 |
| `name` | 显示名 |
| `kind` | `local` / `wsl` / `git-bash` / `msys` / `cygwin` / `ssh` / `serial` / `container` |
| `program`, `args` | 启动程序与参数；`kind` 可提供默认配方 |
| `distro`, `user`, `host`, `port` | 远程/发行版信息 |
| `cwd` | `home` / `session` / `fixed` / `last` / 绝对路径 |
| `env` | 额外环境变量 |
| `color`, `icon`, `badge` | 视觉标识 |
| `font`, `theme` | 可选覆盖 |
| `startup` | 启动命令列表（AUTO-01） |
| `shell_integration` | `auto` / `on` / `off` |

### 5.3 会话恢复状态（草案）

```json
{
  "version": 1,
  "windows": [{
    "bounds": [100, 100, 1200, 800],
    "active_tab": 0,
    "tabs": [{
      "name": "api",
      "panes": [
        { "profile": "wsl-arch", "cwd": "/home/u/src/api", "ratio": 0.6 },
        { "profile": "pwsh", "cwd": "C:\\src\\api", "ratio": 0.4 }
      ]
    }]
  }]
}
```

- 默认只存布局、profile、cwd；滚动缓冲不存。
- 可选「最近 1k 行快照」，上限 20MB，加密可选。
- 崩溃恢复用同一格式 + `dirty` 标记。

### 5.4 CLI（草案）

| 参数 | 行为 |
|---|---|
| `-d, --working-directory <dir>` | 新标签的 cwd |
| `-e, --command <cmd> [args...]` | 运行命令；结束是否保留窗口由 `--hold` 决定 |
| `--profile <id>` | 使用指定 profile |
| `--tab` / `--window` | 在当前实例开标签 / 开新窗口 |
| `--title <text>` | 设置标签/窗口标题（清洗后） |
| `--config <path>` | 使用另一份配置 |
| `--hold` | 命令结束后保留会话 |
| `--class <name>` | Linux WM_CLASS |
| `--version` / `--help` | 版本/帮助 |

退出码：正常退出透传子进程退出码；启动失败返回非零并打印原因。

### 5.5 URL scheme（草案）

```text
elyterminal://new?cwd=C%3A%5Csrc&profile=wsl-arch
elyterminal://new?profile=ssh-prod&command=tail%20-f%20app.log&confirm=1
```

- 只接受白名单参数。
- `command` 默认不执行；带 `confirm=1` 时弹确认并显示完整命令。
- 未知参数忽略并记录，不报错退出。

### 5.6 OSC 支持矩阵（摘要）

| 序列 | 现状 | 目标 |
|---|---|---|
| OSC 0/2 标题 | 已有 | 加清洗/限长 |
| OSC 4/10/11/12/104/110–112 | 已有 | 按 pane 隔离、UI 一致 |
| OSC 7 | 已有 | `Location` 映射 |
| OSC 8 | 已有（id） | 参数、白名单 |
| OSC 9 / 777 / 99 | 已有 | 与 ConEmu 区分、聚合 |
| OSC 9;4 | 已有 | 超时清除、UI |
| OSC 52 | 写；读关 | 限长、确认 |
| OSC 133 | A/N/B/I/C/D | k/aid/cl/err、633 |
| OSC 7501 | 已有 | 状态机产品化 |
| OSC 1337 | 缺 | 图片/徽标/变量（延后） |
| OSC 22 | 缺 | 指针形状 |
| OSC 50 | 缺 | 只查询或忽略 |

---

## 6. 质量门槛

### 6.1 性能预算（目标值，按机器调整）

| 指标 | 预算 |
|---|---|
| 冷启动首帧 P95 | < 300ms |
| 热启动首帧 P95 | < 150ms |
| 输入回显 | ≤ 1 帧 |
| 4K 全屏滚动 p99 帧 | < 8ms |
| 100k 行搜索首命中 | < 50ms |
| base RSS | < 80MB |
| 每个隐藏会话 | < 10MB |
| 大粘贴 10MB | UI 不冻结，可取消 |

### 6.2 测试矩阵

| 层 | 内容 |
|---|---|
| 单元 | 路径映射、OSC 解析、按键编码、配置合并、块模型 |
| 集成 | 启动各 shell、WSL/Git Bash、SSH 假服务器、拖放 |
| 协议 | `vttest`、`esctest`、TDN probe、tmux 内外、SSH、ConPTY |
| 模糊 | VT/OSC/CSI 随机字节，不 panic、不越界 |
| 截图 | 固定字体/平台下的帧对比（允许阈值） |
| 手工 | 三平台 × 中文/英文 × HiDPI × 深浅主题 |
| 性能 | criterion + CI 阈值 |
| 安全 | 粘贴、OSC 52、链接、标题、图片解码 |

### 6.3 发布检查

- [ ] `cargo fmt` / `clippy` / 全测试通过。
- [ ] Windows/macOS/Linux 构建。
- [ ] 旧配置能迁移，坏配置能回退。
- [ ] 新功能有开关，默认值安静。
- [ ] 诊断包不含密钥/剪贴板内容。
- [ ] README/`--help`/示例配置一致。

---

## 7. 明确不做（以及为什么）

| 不做 | 原因 |
|---|---|
| 内置 mux / 自研 SSH 协议栈 | 与 tmux、OpenSSH 重复；接进程即可 |
| 终端内 AI 聊天、自动执行命令 | 信任与安全成本高；agent 状态才是终端职责 |
| 全量替换 alacritty VT 引擎 | 成本极高；图形/字素可局部 fork |
| RTL / 阿拉伯语整形 | 与目标用户无关；明确声明不支持 |
| 插件运行时（Lua/JS） | 先用 CLI/IPC + 配置覆盖 80% 场景 |
| 云同步 / 账号 / 遥测 | 个人本地工具，零遥测是优点 |
| 自建更新器（未签名） | 维护成本高；提示 + 打开页面即可 |
| 多光标、富文本输出、自建 diff UI | 编辑器与 Git 面板已覆盖 |
| 顶部标签条与树形标签两套并存 | 先做完整树形标签；顶部条仅作为可选视图 |
| **Windows 默认终端接管** | 涉及签名与系统设置，个人 fork 不承诺 |
| 为 1% 场景加常驻轮询 | 用事件驱动（WSL/SSH 状态、文件监听） |
| 全量自建命令历史/补全 | shell 已有；终端只做索引与搜索 |

---

## 8. 脑洞区（明确实验性，不承诺）

这些是「如果有余力，会很有意思」的功能，先记录，不排期。

1. **会话时间线**：把一次会话的命令、耗时、退出码画成一条可缩放的时间线，点击跳转；适合复盘长任务。
2. **两次运行 diff**：同一命令两次输出并排比较，构建/测试回归一眼看出。
3. **输出结构化视图**：检测 JSON/表格/日志级别，提供「美化查看」，但原始文本始终可复制。
4. **端口仪表盘**：当前会话相关端口、转发、监听进程，一键在浏览器打开。
5. **进程树面板**：当前窗格的前台/子进程、CPU/内存、结束/信号。
6. **目录级 runbook**：项目里放一个 `runbook.md`，终端识别并在侧栏列出可执行步骤（每步需确认）。
7. **会话录制/分享**：asciinema 格式录制，一键导出；默认不录。
8. **协作终端**：只读分享一个会话给另一台机器（WebRTC/中继），权限与审计先设计。
9. **命令「时光机」**：滚动到任意历史命令时，显示当时的 cwd、git 分支、环境摘要。
10. **Ely 桌面伴侣**：独立小窗，按 agent 状态做表情；点击跳回会话。
11. **截图模式**：一键隐藏 UI/宠物，导出干净的终端截图，便于写文档。
12. **本地统计**：最常用命令/目录/时段，只存本机；不做排行榜、不上传。
13. **主题工坊**：在设置里调 16 色并实时预览，导出 iTerm/Ghostty/WT 格式。
14. **终端内图片画廊**：kitty graphics 实现后，`ls` 图片目录显示缩略图（需显式开启）。
15. **键盘提示层**：按住修饰键时在 UI 上标出可用快捷键（类似编辑器）。
16. **智能目录跳转**：解析输出里的路径，`Ctrl+click` 直接在 Files 面板定位（不是 cd）。
17. **命令失败聚合**：一次构建/测试的失败项汇总成一个可跳转列表。
18. **AI 审批收件箱**：所有 agent 的等待审批集中在一个列表，批量处理（仍需逐个确认）。
19. **工作区模板**：`rust-api`、`ssh-ops` 等一键开出标签/窗格/命令/布局。
20. **终端「健康检查」**：启动时检查 shell 集成、字体、WSL、SSH、配置错误，给一份可操作的报告。

---

## 附录 A：本机探测命令

```powershell
# 已安装的 shell
Get-Command bash, wsl, pwsh, powershell, cmd, git, nu, elvish, xonsh -ErrorAction SilentlyContinue

# WSL 发行版（输出为 UTF-16LE）
wsl.exe -l -v

# Git Bash 的真实位置
Get-ChildItem 'C:\Program Files\Git\bin\bash.exe','C:\Program Files\Git\usr\bin\bash.exe'

# 注册表：WSL 发行版
Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss' |
  ForEach-Object { Get-ItemProperty $_.PSPath } |
  Select-Object DistributionName, Version, DefaultUid

# 注册表：Git for Windows
Get-ItemProperty 'HKLM:\SOFTWARE\GitForWindows' -ErrorAction SilentlyContinue

# SSH 生效配置
ssh -G <host>
```

## 附录 B：转义序列速查

| 用途 | 序列 |
|---|---|
| 标题 | `OSC 2 ; title ST` |
| cwd | `OSC 7 ; file://host/path ST` |
| 超链接 | `OSC 8 ; id=… ; URI ST` … `OSC 8 ; ; ST` |
| 通知 | `OSC 9 ; text ST`、`OSC 777 ; notify ; title ; body ST`、`OSC 99 ; i=…:p=title ; payload ST` |
| 进度 | `OSC 9 ; 4 ; state ; percent ST` |
| 剪贴板 | `OSC 52 ; c ; base64 ST`（读为 `?`） |
| 提示符标记 | `OSC 133 ; A/B/C/D ; exit ST` |
| 程序状态 | `OSC 7501 ; state=…:kind=…:app=…:progress=…:msg=b64 ST` |
| 同步输出 | `CSI ? 2026 h` … `CSI ? 2026 l` |
| 括号粘贴 | `CSI ? 2004 h` … `CSI 200~ text CSI 201~` |
| kitty 键盘 | `CSI > flags u` push、`CSI < n u` pop |
| 设备属性 | `CSI c` → `CSI ? … c` |
| 版本 | `CSI > 0 q` → `DCS > \| name(version) ST` |

## 附录 C：参考

- xterm control sequences：<https://invisible-island.net/xterm/ctlseqs/ctlseqs.html>
- kitty graphics / keyboard / notifications：<https://sw.kovidgoyal.net/kitty/graphics-protocol/>、<https://sw.kovidgoyal.net/kitty/keyboard-protocol/>
- Ghostty VT：<https://ghostty.org/docs/vt>
- iTerm2 escape codes：<https://iterm2.com/documentation-escape-codes.html>
- WezTerm escape sequences：<https://wezterm.org/escape-sequences.html>
- Windows Terminal shell integration：<https://learn.microsoft.com/windows/terminal/tutorials/shell-integration>
- WSL 基础命令：<https://learn.microsoft.com/windows/wsl/basic-commands>
- tmux `allow-passthrough`：`man tmux`
- 终端兼容性表：<https://toppk.github.io/revenant/tdn/>
- alacritty_terminal 0.26：`~/.cargo/registry/src/*/alacritty_terminal-0.26.0/`

## 附录 D：与其它文档的关系

- `PRD.md`：已承诺的行为契约；新增功能先改 PRD，再写代码。
- `FEATURES.md`：竞品缺口分析；标签/分屏/reflow 部分已过时，以本文 §1 为准。
- `PLAN.md`：早期里程碑；顶部标签条、reflow、Lua 配置与现状有出入，执行以代码与本文为准。
- `TASKS.md`：任务清单；本文的 ID 可直接拆成任务。
- `future.md`（本文）：功能库与规格草案，不作为验收标准；验收以 PRD 与测试为准。
