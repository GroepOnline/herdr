# CHEF Fleet Operations Layer — Capability Matrix

Baseline: upstream v0.7.3 (OnlineChefGroep/herdr @ f36d804)
Downstream: OnlineChefGroep/herdr @ master (v0.7.3-chef)
Date: 2026-07-12

## 1. Agent Detection & Semantic State

| Capability | Upstream | Fork | Status |
|---|---|---|---|
| Semantic states (Idle/Working/Blocked/Unknown) | YES | = | Native |
| Detection manifests | 18 built-in | +3 (freebuff, junie, openclaude) | Extended |
| Remote manifest updates | YES (herdr.chefgroep.nl) | Disabled via HERDR_UPDATE_BASE_URL | Isolated |
| Agent labels on pane borders | YES | Config: show_agent_labels_on_pane_borders=false | Configured |
| OSC title-based detection | YES | = | Native |
| Regex/contains rule engine | YES (min_engine_version=2) | = | Native |
| Amp detection | YES | = | Native |
| Grok Build detection | YES (v0.2.82 patterns) | = | Native |

## 2. Rendering & Terminal

| Capability | Upstream | Fork | Status |
|---|---|---|---|
| Kitty graphics protocol | YES | +sync wrapping (ESC 7/8) | Extended |
| Synchronized output (DEC mode 2026) | YES | = | Native |
| ANSI render encoding | SemanticFrame default | TerminalAnsi default | **Modified** |
| IME anchor repeat after sync | YES (conditional) | Disabled (client-safe) | **Modified** |
| Wide cell row boundary clamping | NO | YES (#8 fix) | **New** |
| Zero-width char safety net | NO | YES (#10 fix) | **New** |
| Border-title style reset | NO | YES (#11 fix) | **New** |
| Cursor memory between frames | YES | = | Native |
| Hyperlink support (OSC 8) | YES | = | Native |
| Profiling (render_prof) | YES | = | Native |

## 3. Session & Persistence

| Capability | Upstream | Fork | Status |
|---|---|---|---|
| Atomic session snapshots | YES | = | Native |
| Session restore (shell-only) | YES | = | Native |
| Native agent-session restore | YES | = | Native |
| Snapshot history | YES | = | Native |
| Pane zoom state in snapshots | YES | = | Native |
| Worktree support | YES | = | Native |
| Copy mode (vim/emacs) | YES | = | Native |
| Auto-reconnect with backoff | **NO** | Phase 3 target | Gap |
| Heartbeat/stale detection | Stale socket only | Phase 3 target | Gap |
| Restore verification | Partial | Phase 3 target | Gap |
| Recovery audit log | **NO** | Phase 3 target | Gap |

## 4. Plugins & Integration

| Capability | Upstream | Fork | Status |
|---|---|---|---|
| Plugin registry | YES | = | Native |
| Plugin manifest reload | YES | = | Native |
| Plugin context merge | YES | = | Native |
| OpenCode integration | YES (plugin install) | = | Native |
| Linear integration | **NO** | Phase 5 target | Gap |
| GitHub PR/CI status | **NO** | Phase 5 target | Gap |
| Cloudflare tunnel status | **NO** | Phase 5 target | Gap |
| MCP bridge | **NO** | Phase 5 target | Gap |
| Issue-to-workspace provisioning | **NO** | Phase 5 target | Gap |

## 5. API & Networking

| Capability | Upstream | Fork | Status |
|---|---|---|---|
| Unix socket API | YES | = | Native |
| API schema (JSON-RPC style) | YES | = | Native |
| Event subscriptions | YES | = | Native |
| HTTP/REST gateway | **NO** | Phase 4 target | Gap |
| WebSocket/SSE events | **NO** | Phase 4 target | Gap |
| Health endpoint | **NO** | Phase 4 target | Gap |

## 6. Config & Update

| Capability | Upstream | Fork | Status |
|---|---|---|---|
| TOML config | YES | = | Native |
| Config profile import/export | YES | +chef.toml profile | Extended |
| Update channel (stable/preview) | YES | +HERDR_UPDATE_BASE_URL override | Extended |
| Auto-update from herdr.chefgroep.nl | YES | Disabled (isolated) | **Modified** |
| Hot config reload | Plugin manifests only | Phase target (not in scope) | Gap |
| Sound config | YES (SoundConfig) | = | Native |
| Toast notifications | YES (terminal/herdr/clipboard) | = | Native |

## 7. Fleet Operations (ALL GAPS)

| Capability | Status |
|---|---|
| Fleet operations bar per pane | **Phase 2 — does not exist** |
| Agent metadata (repo/branch/PR/model/host) | **Phase 2 — does not exist** |
| Linear issue context | **Phase 5 — does not exist** |
| GitHub PR/CI status | **Phase 5 — does not exist** |
| Session parking | **Phase 5 — does not exist** |
| Pi-Helios lifecycle integration | **Phase 6 — does not exist** |
| systemd user-service | **Phase 3 — does not exist** |

## 8. Superseded Roadmap Items

The following items from our earlier brainstorm are **superseded by upstream** and will NOT be built:

| Item | Reason |
|---|---|
| Agent sounds | Already implemented: SoundConfig with state-change triggers |
| Pane zoom | Already implemented: zoom state tracked in snapshots/restore |
| Config hot-reload | Plugin manifest reload exists; full config reload is upstream's responsibility |
| Amp detection manifest | Already in upstream |
| Remote manifests | Already implemented: background herdr.chefgroep.nl check |
| OpenCode bridge | Already integrated: plugin install to opencode plugins dir |
| Session restore | Already implemented: native agent-session restore + snapshot history |
| Generic agent status bar | Upstream has border labels + semantic state; we build Fleet Ops Bar instead |

