# Batches from future.md (P0 + a part of P1)

Each feature: one commit. Each group: one push. Tests run before each commit,
`scripts/check.sh` before each push. Docs (`future.md` §1.4, `TASKS.md`) at each group's end.

## Group 1 — 安全默认值与能力协商 (5)

1. SEC-05 标题清洗：OSC 0/2 去 C0/C1 与 BiDi 控制符，限长。
2. SEC-04 链接白名单：只有 http/https/mailto 可点。
3. PROTO-01 DA1：按真实能力回答 `CSI c`，不报 VT102。
4. PROTO-02 XTVERSION：`CSI > 0 q` → `DCS > | ELYTerminal(0.1.7) ST`。
5. PROTO-03 DECRQM：`CSI ? Ps $ p` 按真实模式表回答，未知答 0。

## Group 2 — 启动诊断与搜索 (5)

6. SHELL-13 启动失败诊断：保留 stderr、退出码、尝试过的路径，可复制。
7. FIND-02 搜索选项：大小写 / 整词 / 正则。
8. FIND-05 结果计数与导航：`3/17`、循环、Esc 回底。
9. FIND-13 正则错误提示：就地显示错误位置。
10. FIND-06 已完成（滚动条标记）；改做 SEL-12 复制路径：当前目录 / 选中路径。

## Group 3 — 标签、窗格与配置 (5)

11. TAB-15 分屏比例：拖动分隔条、双击均分、持久化。
12. TAB-13 克隆标签：同 profile 同 cwd 开新标签。
13. CFG-16 生成带注释的默认 `config.toml`。
14. CFG-06 环境变量插值：`${HOME}` / `${USER}`。
15. PATH-08 复制路径格式：Windows / POSIX / WSL / URI。

## Group 4 — 命令块 (5)

16. BLOCK-03 复制命令 / 输出。
17. BLOCK-05 块导航：上一条 / 下一条 / 上一条失败。
18. BLOCK-10 导出块：文本 / Markdown / JSON。
19. BLOCK-11 命令历史索引：按退出码与耗时统计。
20. BLOCK-06 块 gutter 状态色：运行中 / 成功 / 失败。

## Group 5 — 协议与状态 (5)

21. PROTO-10 47/1047/1048 旧备用屏与保存光标。
22. PROTO-13 OSC 133 扩展：`k`、`aid`、`cl`、`err`；633 兼容。
23. STAT-14 状态栏：连接、TERM、尺寸、前台进程。
24. STAT-17 Git 分支徽标。
25. CONN-18 连接状态徽标：WSL 发行版、user@host。
