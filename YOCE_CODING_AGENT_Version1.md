# Yoce Agent Browser — 架构契约

> Source of truth for coding agents.  
> Last updated: 2026-05-13  (D.1 completed — AgentPanel skeleton)

---

## 1. 项目定位

Yoce 是 **AI Agent 浏览器**。核心价值是 agent runtime，不是多引擎支持。

**唯一引擎**：wry (Windows: WebView2, macOS: WKWebView)。  
**唯一 Shell**：gpui。  
**共享契约**：yoce-engine 定义 ShellCommand / ShellEvent / 共享类型。

---

## 2. Crate 职责

| Crate | 职责 |
|-------|------|
| yoce-app | gpui shell、WebView 宿主、命令路由 |
| yoce-engine | ShellCommand、ShellEvent、共享错误类型（纯数据，零运行时依赖） |
| yoce-agent-runtime | agent 运行时（未来） |

---

## 3. 不可违背的规则

1. **yoce-engine 是唯一共享契约**：shell 和 agent 通过 ShellCommand / ShellEvent 通信
2. **gpui 拥有事件循环**：wry 作为子窗口嵌入，不抢占事件循环
3. **所有操作通过 dispatch()**：UI 按钮和 agent 命令走同一条路径
4. **零 unsafe**：除非引擎平台 API 要求

---

## 4. 当前架构

```
yoce-app/src/
├── main.rs
├── agent/
│   ├── mod.rs
│   ├── panel.rs         AgentPanel 聊天 UI (D.2)
│   └── store.rs         AgentStore 共享消息存储 (D.3)
├── components/
│   └── button.rs
├── shell/
│   ├── mod.rs           build_root, UnsupportedShell, 工具函数
│   ├── yoce_shell.rs    YoceShell + dispatch() + Render
│   ├── polling.rs       NavState 后台轮询
│   └── keyboard.rs      键盘/地址栏事件
└── state/
    ├── tab_state.rs     TabState
    └── nav_state.rs     NavState (Arc<Mutex<>>)
```

**Shell 直接使用 wry**（不再通过抽象层），YoceShell 通过 `yoce_engine::ShellCommand` 和 `yoce_engine::ShellEvent` 与外部通信。

---

## 5. 执行阶段

### Phase A — Shell 核心 ✓
嵌入 webview、地址栏、标签页、跨平台、新窗口拦截、URL/title 跟踪、后台轮询。

### Phase B — 模块化 ✓
shell.rs 拆为 shell/ + state/ + components/。

### Phase C — 统一命令层 ✓
ShellCommand + ShellEvent 移入 yoce-engine。dispatch() 为 UI 和 agent 的统一入口。

### Phase D — Agent Runtime ← current
agent 加载/运行/卸载，通过 ShellCommand/ShellEvent 与 shell 通信。

子步骤：
- **D.1: AgentPanel 骨架 ✓**
- **D.2: AgentPanel 聊天 UI ✓**（消息列表 + 输入框 + Send 按钮）
- **D.3: AgentStore 状态管理 ✓**（独立 Entity、共享消息存储）
- **D.4: 统一日志系统 ✓**（LogStore + log crate + env_logger）
- D.5: AgentEvent 观察者（订阅 ShellEvent）
- D.6: Agent 运行时集成（native/WASM）

### Deferred
- 多引擎支持（CEF 等）
- Extension host
- RL environment

---

## 6. 构建命令

```
cargo build -p yoce-app
cargo clippy -p yoce-app -- -D warnings
```

---

## 7. Done 标准

1. build + clippy 通过
2. shell 和 agent 的通信走 yoce-engine 契约
3. 不退化现有 shell 行为
4. 本文档同步更新
