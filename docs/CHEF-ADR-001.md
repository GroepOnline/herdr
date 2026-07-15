# ADR-001: CHEF Fleet Operations Layer

Date: 2026-07-12
Status: Accepted

## Context

OnlineChefGroep/herdr is a downstream fork of OnlineChefGroep/herdr v0.7.3.
Upstream provides terminal multiplexing, AI agent detection, session
snapshots, and plugin infrastructure. We need operational fleet
capabilities (agent metadata, crash recovery, API gateway, Linear/GitHub
integration) that upstream does not provide.

## Decision

Build a **CHEF Fleet Operations Layer** as a series of additive modules
on top of upstream, without modifying upstream core behavior except
where already patched (rendering, update isolation).

### Architecture

```
┌─────────────────────────────────────────────┐
│  Fleet Ops Bar (Phase 2) — per-pane metadata │
├─────────────────────────────────────────────┤
│  Crash Recovery (Phase 3) — systemd + reconnect│
├─────────────────────────────────────────────┤
│  API Gateway (Phase 4) — localhost HTTP/SSE   │
├─────────────────────────────────────────────┤
│  CHEF Plugins (Phase 5) — Linear/GitHub/CF    │
├─────────────────────────────────────────────┤
│  Integrations (Phase 6) — Pi-Helios/Aider     │
├─────────────────────────────────────────────┤
│  Upstream Herdr v0.7.3 Core (unmodified)      │
├─────────────────────────────────────────────┤
│  Downstream Patches (rendering, update isol.) │
└─────────────────────────────────────────────┘
```

### Principles

1. **Upstream semantic state is authority.** Fleet Ops metadata
   supplements but never overrides AgentState (Idle/Working/Blocked).

2. **Linear is SSOT for work status.** GitHub is SSOT for code/PR/CI.
   Host/Cloudflare APIs are SSOT for runtime. No new combined
   stale source of truth.

3. **Read-only by default.** API gateway serves read endpoints without
   auth. Mutations require explicit scopes.

4. **No herdr.chefgroep.nl runtime dependency.** Update channel isolated via
   HERDR_UPDATE_BASE_URL. No runtime downloads from upstream.

5. **Claim nothing about reboot survival.** Crash recovery restores
   shells, cwd, worktree, and session metadata. Agent process
   continuity is best-effort, not guaranteed.

### What we do NOT build

- GPU rendering, Sixel, Unicode version projects
- New plugin framework (use upstream's existing registry)
- Generic OpenCode bridge (upstream already integrates)
- Screenshots as core feature

## Consequences

- Each phase produces tests, schema definitions, migration path
- All code stays within the fork repository
- Upstream rebase must preserve downstream patches
- Plugin repositories are separate from core fork

