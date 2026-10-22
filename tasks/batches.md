# Batches from future.md

Each feature: one commit. Each group: one push after `cargo test`, `cargo clippy -D warnings`,
`cargo fmt --check`, the 500-line rule and the one-line-comment rule. Docs (`future.md` §1.4 and
`TASKS.md`) at each group's end.

## Group 1 — 安全默认值与能力协商 (5 commits, pushed)

1. SEC-05 标题清洗：OSC 0/2 去 C0/C1 与 BiDi 控制符，限长 1024。
2. SEC-04 链接白名单：只有 http/https/mailto/file 可点。
3. PROTO-01 / 02 / 03：DA1 答 `62;22`，XTVERSION 答 `ELYTerminal(版本)`，DECRQM 按真实模式表回答。
4. SHELL-13：启动失败保留程序、目录、错误与前 8 行程序输出，窗口里可复制。

## Group 2 — 路径、设置与克隆 (5 commits, pushed)

5. PATH-08 / PANEL-09：路径可按系统 / Git Bash / WSL / 引号 / URI 五种形式复制。
6. CFG-06：`${VAR}` 从环境变量展开，不执行命令。
7. TAB-13：`Ctrl/Cmd-Shift-K` 克隆会话到新标签，同程序同目录。
8. CFG-16：首次运行写出带注释的 `config.toml`。
9. 文件拆分，保持每份源码在 500 行以内。

## Group 3 — 命令块、协议与状态 (7 commits, pushed)

10. BLOCK-03 / 05 / 10 / 11、STAT-19：命令文本捕获、块导航、失败列表、导出（文本 / Markdown / JSON）。
11. PROTO-13：OSC 633 与 133 同读，`aid` / `err` 等选项不再作废标记。
12. PROTO-10：47 / 1047 / 1048 改写成 1049 交给解析器。
13. STAT-17：终端列标题显示 Git 分支。
14. STAT-14 / CONN-18：状态行（连接、列数、分支）与会话行上的连接前缀。

## Group 4 — 分层设置 (2 commits, pushed)

15. CFG-05：`include` 多文件叠放，成环即停，缺失点名。
16. 种子文件说明 `include`。

## Group 5 — 健康检查 (2 commits, pushed)

17. future.md §8 第 20 条：`Ctrl-Shift-Alt-H` 报告设置、集成脚本与每台 shell 的状况，叠在终端上并可复制。

## Group 6 — 窗格与标签的人体工学 (2 commits, pushed)

18. TAB-15：窗格宽度与行高改成权重，布局按份额伸缩；行内与行间的边界可拖动，双击或 `Ctrl-Shift-Alt-E` 重新均分，最小 5%。
19. TAB-03 / TAB-04：`Ctrl-Shift-Alt-PageUp/PageDown` 移动标签，`Ctrl-Shift-Alt-P` 固定，固定的排在前面并显示 `★`。

## Group 7 — 安全与导航 (1 commit, pushed)

20. SEL-08 / SEC-03 / PROTO-16：OSC 52 写入超过 100 KiB 拒绝并记日志。
21. STAT-19 / FIND-08：会话悬停列出失败过的命令；`Ctrl-Shift-Alt-↑/↓` 跳到上一条/下一条失败命令。
22. INP-14（部分）：快捷键表补全可搜索，`Ctrl-Shift-/` 打开。

## Group 8 — 窗口标题 (1 commit, pushed)

23. UI-17：标题模板 `{title}` / `{cwd}` / `{shell}`，设置里有输入框，种子配置里有说明。

## 未做（留给下一轮）

`PERF-01/02/10`（行级 damage、shaping 缓存、性能面板）需要改渲染核心，`CFG-09`（键位自定义）
需要一张 action 表与录制 UI，`TAB-22`（多窗口）与 `TAB-23`（快速终端）需要平台窗口 API。
这几块风险最大，应各自单独一轮，配人工验收。
