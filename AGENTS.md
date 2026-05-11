# Yoce — Coding Agent 速查

> 详细架构：`YOCE_CODING_AGENT_Version1.md`  
> 本文是日常执行指南。

## 1. 阅读顺序

1. 先看架构契约（`YOCE_CODING_AGENT_Version1.md`）
2. 再看本文
3. 检查当前阶段状态

## 2. 项目定位

**Yoce = AI Agent 浏览器。**

- 引擎：wry (WebView2 / WKWebView)  
- Shell：gpui  
- 共享契约：yoce-engine (ShellCommand + ShellEvent)

## 3. 快速校验

```bash
cargo build -p yoce-app
cargo clippy -p yoce-app -- -D warnings
```

## 4. 硬规则

1. wry 唯一直连 yoce-app shell，不需要 engine trait 抽象层
2. gpui 拥有事件循环
3. 所有操作通过 `YoceShell::dispatch()` 路由
4. Shell 和 agent 通过 `yoce_engine::ShellCommand` / `ShellEvent` 通信
5. 每次改动后必须能编译，不积累跨阶段的改动

## 5. 当前架构

```
yoce-app/src/
├── main.rs
├── agent/
│   └── panel.rs             AgentPanel 骨架
├── components/button.rs
├── shell/
│   ├── mod.rs               build_root, UnsupportedShell, 工具函数
│   ├── yoce_shell.rs        YoceShell + dispatch() + Render
│   ├── polling.rs           后台轮询
│   └── keyboard.rs          键盘/地址栏事件
└── state/
    ├── tab_state.rs
    └── nav_state.rs         跨线程共享状态
```

## 6. 当前阶段

| Phase | 状态 |
|-------|------|
| A — Shell 核心 | ✓ |
| B — 模块化 | ✓ |
| C — 统一命令层 | ✓ |
| D — Agent Runtime | ← current |

## 7. 实现模式

1. 选一个垂直切片
2. 最小代码完成
3. build + clippy 通过
4. 架构变了就同步更新文档

## 8. Done 标准

1. build + clippy 通过
2. 无退化
3. 文档已更新
