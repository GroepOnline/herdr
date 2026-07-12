# Phase 6 — Integrations Design

## Priority 1: Pi-Helios Lifecycle Integration

**Existing infrastructure:** Herdr already has full Pi integration via `src/integration/assets/pi/herdr-agent-state.ts` — Unix socket IPC with newline-delimited JSON. Events: session_start, agent_start, agent_end, session_shutdown, herdr:blocked. Hook authority: `("herdr:pi", "pi")`.

**Pi-Memory architecture:** File-based (SQLite + MEMORY.md). CLI: `pi-memory log/query/search/state/sync`.

**What to build:**
- Extend `herdr-agent-state.ts` to call `pi-memory` CLI on session lifecycle
- Add Pi-specific resume in `agent_resume.rs` (populate context from `pi-memory query`)
- Track task state via `pi-memory state <project>`

**Effort:** 3-5 days. Risk: verify Pi-Helios-Memory CLI interface matches upstream Pi-Memory.

## Priority 2: Aider Detection

**Status:** DONE — `src/detect/manifests/aider.toml` created with:
- `yes_no_confirmation` (blocked) — matches `(y)es/(n)o` confirmation prompts
- `spinner_working` (working) — matches Aider's ░█ bounce spinner + "waiting for llm"

**Code changes:** Agent::Aider variant added to enum, agent_label, parse_agent_label, SCREEN_MANIFEST_AGENTS, BUNDLED_MANIFESTS.

**No lifecycle integration possible** — Aider has no hooks system (confirmed via GitHub issue #2557). Screen detection only.

## Priority 3: Continue.dev — SKIP

**Rationale:**
1. Acquired by Cursor — project winding down
2. No identifiable terminal markers (ink-based TUI, no stable text patterns)
3. No hooks system
4. Low fleet CLI adoption

## Priority 4: Amp + OpenCode — Already Covered

| Agent | Detection | Lifecycle | Action |
|---|---|---|---|
| Amp | ✅ `amp.toml` (2 rules) | ❌ No hooks | None needed |
| OpenCode | ✅ `opencode.toml` (3 rules) | ✅ v8 plugin + hook authority | None needed |

## NOT Building (per spec)

- GPU rendering
- Sixel
- Unicode version project
- New plugin framework
- Generic OpenCode bridge
- Screenshots as core feature

