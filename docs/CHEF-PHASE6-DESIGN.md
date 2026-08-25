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

**Status:** SHIPPED (screen detection) — landed on `main` as #75 (`7e652729`). Current `main` has `Agent::Aider`, bundled `src/detect/manifests/aider.toml`, `SCREEN_MANIFEST_AGENTS` registration, parse/process lookup (`aider` / `aider-cli`), website `agent-detection/aider.toml`, and a `docs/next/README.md` listing.

The deleted `feat/aider-integration` branch was a different, partial tree and must not be revived or cherry-picked. The previous DONE note in this doc was also stale in a second way: it described `yes_no_confirmation` / `spinner_working` rules that are not what shipped.

**Shipped rules:**
- `thinking_working` (working)
- `prompt_idle` (idle)
- `file_edit_done` (idle)
- `error_detected` (blocked; specific API/rate-limit/connection/timeout strings, not bare `error:`)

**Remaining after detection:**
- Capture live Aider terminal states and process evidence before treating detection quality as finished.
- Keep error matching narrow; do not add generic whole-buffer `error:` rules.
- Screen detection only — Aider has no hooks system (GitHub issue #2557).

**Historical intent only:** branch `feat/aider-integration` @ `a2ba5194` (deleted); do not cherry-pick.

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

