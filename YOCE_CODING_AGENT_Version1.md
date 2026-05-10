
# Yoce Agent Browser - Architecture Spec

> Project: Yoce Agent Browser
> Role: Source of truth for architecture, agent capability, and execution order
> Last Updated: 2026-05-11

---


## 1. Product Vision

Yoce is an AI Agent Browser designed for agent-driven automation, extensibility, and multi-engine embedded browsing. The core design principles are:

- gpui owns the app shell and UI lifecycle.
- Browser engines plug in through shared contracts.
- Browsing is always in-app, never external.
- Agent runtime is a first-class, pluggable component.
- CEF and WebView2 are both first-class engine targets.
- All features are designed for agent and UI parity.

---


## 2. Current Baseline

Minimal workspace for rapid iteration:

- yoce-app/src/main.rs
- yoce-app/src/shell.rs
- yoce-engine/src/lib.rs

Current status:

- yoce-app builds and runs with embedded webview, address bar, and in-memory tab model.
- yoce-engine defines shared engine contracts.

---


## 3. Target Product Definition

Yoce is a browser where AI Agents are first-class citizens, able to:

1. Access and automate all browser features (navigation, tabs, DOM, etc.).
2. Observe and interact with tab/page state via stable APIs.
3. Run in a secure, extensible runtime (native or WASM).
4. Share a unified command surface with the UI.
5. Support multiple browser engines with strict shell/engine separation.

---


## 4. Crate-Level Architecture Plan

| Crate                | Responsibility                                 | Priority |
|----------------------|------------------------------------------------|----------|
| yoce-app             | gpui shell, app lifecycle, command orchestration, agent bridge | P0 |
| yoce-engine          | shared traits, types, tab/page abstractions     | P0 |
| yoce-engine-webview2 | Windows embedded engine implementation          | P1 |
| yoce-engine-cef      | Chromium implementation                        | P1 |
| yoce-agent-runtime   | agent lifecycle, runtime (native/WASM), tool mapping | P1 |
| yoce-extension-host  | extension host (deferred)                      | P3 |
| yoce-rl-env          | RL environment (deferred)                      | P3 |
| yoce-protocol-handlers | protocol handlers (deferred)                  | P3 |

---


## 5. Non-Negotiable Invariants

1. Engine isolation
	- yoce-app uses only yoce-engine traits.
	- No engine-specific types in shell orchestration.
2. Main-thread UI ownership
	- gpui shell owns window and visual lifecycle.
3. No reentrant mutable borrows across engine operations
	- Never hold mutable shell state borrows while calling engine operations that may reenter callbacks.
4. Engine-agnostic bounds path
	- Embedded content placement uses shared methods (resize, set_bounds).
5. Fallible commands
	- All navigation, tab, JS, and automation commands are fallible.
6. Agent/shell parity
	- All browser features must be accessible to both UI and agent runtime via the same command layer.

---


## 6. Shared Contracts (Reference)

yoce-engine defines the following minimal contracts:

- Engine
- BrowserEngine
- WebViewHandle
- EngineEvent
- LoadStatus
- EngineError

Rules:
1. Contract changes must be justified by a real shell or agent use case.
2. All engine-specific additions must be implementable by both WebView2 and CEF.
3. Prefer additive over breaking changes.

---


## 7. Execution Plan (Phases)

Follow these phases. Do not skip steps.

### Phase A - Shell & Engine Core
Goal: Stable shell with in-app webview, address bar, tabs, and engine abstraction.
Validation: cargo build -p yoce-app

### Phase B - Shell Modularization
Goal: Split shell logic (state, rendering, address edit) for maintainability.
Validation: cargo build -p yoce-app; cargo clippy -p yoce-app -- -D warnings

### Phase C - Unified Command Layer
Goal: All navigation/tab/page actions routed through a command bus, shared by UI and agent runtime.
Validation: cargo build -p yoce-app; focused unit tests for command dispatch

### Phase D - Engine Implementations
Goal: Move WebView2/CEF details behind yoce-engine-webview2/cef crates.
Validation: cargo build -p yoce-app --features engine-webview2/cef

### Phase E - Agent Runtime & API
Goal: Integrate agent runtime (native/WASM), expose tab/page info and automation APIs to agents, ensure all browser features are accessible via command layer.
Validation: Agents can run, access tab/page info, and automate browser via API.

---


## 8. Definition Of Done

Every architecture-affecting change must:
1. Pass build for affected crate(s).
2. Update this file if boundaries, phases, or invariants change.
3. Preserve shell/engine and agent/shell isolation.
4. Not regress current shell or agent baseline behavior.

---


## 9. Coding Agent Workflow Rules

1. Read this file first, then AGENTS.md.
2. Choose the smallest vertical slice that moves the current phase forward.
3. Keep edits scoped; avoid broad refactors without phase justification.
4. Validate after each slice.
5. If blocked, document the blocker and next action.
6. When unsure, prefer options that maximize engine-agnostic and agent-accessible design.

---


## 10. Build And Validation Commands

Primary:
- cargo build -p yoce-app
- cargo test -p yoce-app
- cargo clippy -p yoce-app -- -D warnings

Engine-specific:
- cargo build -p yoce-app --features engine-webview2
- cargo build -p yoce-app --features engine-cef

Policy:
- Validate narrow scope first.
- Do not run broad workspace checks unless required.

---


## 11. Deferred Scope

Defer until browser core and agent runtime are stable:
- Extension host
- RL environment
- Protocol handler
- Advanced automation/agent orchestration

---


## 12. Quick Decision Checklist

Before merging a change, answer yes to all:
1. Does this help the current phase objective?
2. Does this preserve shell-engine and agent/shell isolation?
3. Can both WebView2 and CEF follow the same shell path?
4. Is the change validated with required build command(s)?
5. If architecture changed, did this document get updated?

---

## 13. Agent Capabilities & Runtime (Key Planning)

### 13.1 Agent Access to Tab/Page Information

Agents must be able to:
- Enumerate all open tabs and their metadata (id, title, url, favicon, load state, etc.).
- Query the active tab and its state.
- Access page content (DOM snapshot, text, selected elements, etc.) via a stable API.
- Subscribe to tab/page events (navigation, load, error, DOM mutation, etc.).
- Request actions (navigate, reload, close tab, execute JS, etc.) through the command layer.

**Proposed API surface (yoce-engine/yoce-app):**
- `TabInfo` struct: { id, title, url, favicon, load_state, ... }
- `PageInfo` struct: { dom_snapshot, selection, scroll, ... }
- `AgentCommand` enum: { Navigate, Reload, CloseTab, ExecuteJs, ... }
- `AgentEvent` enum: { TabCreated, TabClosed, PageLoaded, DomChanged, ... }

All agent access must go through the same command/event bus as the UI, ensuring parity and isolation.

### 13.2 Agent Runtime Design

Agents may run in one of two modes:
- **Native:** Compiled Rust (or other) code loaded as a plugin or process.
- **WASM:** WebAssembly modules loaded and executed in a sandboxed runtime.

**WASM as Agent Runtime:**
- Pros: Secure, portable, language-agnostic, easy to sandbox.
- Cons: Limited host API surface, more complex integration for async/evented APIs.

**Agent Runtime Requirements:**
- Must support loading, running, and unloading agents at runtime.
- Must expose a stable API for agents to access tab/page info and issue commands.
- Must isolate agent execution (no direct memory access to shell/engine state).
- Must allow both UI and agent to issue commands through the same pipeline.

**yoce-agent-runtime** will provide:
- Agent lifecycle management (load, run, stop, unload).
- Host API for tab/page info, events, and commands.
- WASM runtime integration (using wasmtime or similar), with a clear FFI boundary.
- Native agent support (optional, for trusted/first-party agents).

**Open Questions:**
- How much of the browser API surface is exposed to WASM agents initially?
- What is the minimal set of events and commands required for useful automation?
- How are agent permissions and resource limits enforced?

---

**Summary:**
Yoce is an agent-centric browser. All browser state and actions are accessible to agents via a stable, engine-agnostic API, with a pluggable agent runtime (WASM preferred for isolation). All new features must be designed for agent and UI parity, and validated through the shared command/event layer.
