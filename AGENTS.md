# Yoce Agent Browser - Coding Agent Playbook

> Source of truth: YOCE_CODING_AGENT_Version1.md
> This file: short operational guide for day-to-day coding agent execution

## 1. Read Order

Before coding:

1. Read YOCE_CODING_AGENT_Version1.md
2. Read this AGENTS.md
3. Inspect current phase and current baseline files

If this file conflicts with the architecture spec, the architecture spec wins.

## 2. Mission

Rebuild Yoce into a gpui-shell browser with pluggable engines.

Immediate focus:

- stable shell behavior in yoce-app
- shared contracts in yoce-engine
- no engine-specific leakage into shell orchestration

## 3. Fast Commands

Use narrow validation first:

- cargo build -p yoce-app
- cargo test -p yoce-app
- cargo clippy -p yoce-app -- -D warnings

Engine checks:

- cargo build -p yoce-app --features engine-webview2
- cargo build -p yoce-app --features engine-cef

## 4. Hard Rules

1. Do not restore legacy architecture just because it existed.
2. Keep gpui as the only shell owner in yoce-app.
3. Keep yoce-app engine-agnostic through yoce-engine traits.
4. Keep one shared shell command path for all engines.
5. Keep changes phase-oriented and buildable after each slice.

## 5. Current Working Baseline

Repository currently contains minimal crates:

- yoce-app
- yoce-engine

Current implemented shell slice includes:

- embedded webview demo path
- address bar editing path
- in-memory tab behavior path

Treat this as a seed, not final architecture.

## 6. Implementation Pattern

For each task:

1. pick one vertical slice
2. implement minimal code to complete the slice
3. validate with required build command
4. update architecture spec if boundaries or phase plan changed

Do not batch unrelated changes.

## 7. What To Defer

Do not expand these until browser core is stable:

- extension host depth
- RL environment depth
- protocol completeness
- broad agent orchestration features

## 8. Done Criteria

A change is done only if:

1. affected build commands pass
2. shell-engine isolation remains intact
3. architecture spec is updated when architecture changed
4. no regression in current shell baseline behavior
