---
name: chef-fleet
description: CHEF fleet operations for herdr — Linear/GitHub/UDO/Kater SSOT rules, Fleet Ops Bar, plugin state-dir contracts, and when to touch core vs com.chefgroep.* plugins. Use for fleet ops, Linear/GitHub context, Kater/UDO, Cloudflare tunnels, session park, or Fleet/Plugins settings.
---

# CHEF fleet operations

## SSOT (do not invent a fourth)

From `docs/CHEF-ADR-001.md`:

| Domain | Authority |
|--------|-----------|
| Work / issue / cycle / assignee | **Linear** |
| Code / PR / CI | **GitHub** |
| Host / tunnel / runtime health | **Host + Cloudflare APIs** |

`fleet_ops.json` fragments are **aggregates for the TUI bar**, never secrets and never a new SSOT.

## Core vs plugin

| Touch core fork | Keep in plugin (`com.chefgroep.*`) |
|-----------------|-------------------------------------|
| Fleet Ops Bar render + Settings Fleet/Plugins tabs | Linear/GitHub/Kater/UDO/fleet-health/CF/provision/park |
| Merge fragments from plugin state dirs | API tokens in plugin config `.env` (mode 0600) |
| Link handlers / bar hit actions that call plugin actions | Polling, GraphQL, Tailscale probes |

Do **not** put sidebar/widget names on the wire. Bar + settings are TUI/client only.

## State-dir contract

Herdr injects `HERDR_PLUGIN_STATE_DIR` / `HERDR_PLUGIN_CONFIG_DIR`.

```text
$HERDR_PLUGIN_STATE_DIR/
  fleet_ops.json    # fragment for Fleet Ops Bar
  cache/            # optional
  parked/           # session-park only
```

Fragment shape (fields optional): `source`, `updated_at`, `ttl_seconds`, `issue`, `pr`, `fleet`, `cloudflare`, `parked`.

Core merge: `src/fleet/ops.rs` (`load_plugin_fleet_fragments` / `merge_plugin_fragments`).

## Local plugins

Scaffold under `plugins/` (link with `herdr plugin link <path>`):

- `linear-context`, `github-status`, `kater-bridge`, `udo-metrics`
- `fleet-health`, `cloudflare-tunnel`, `issue-provision`, `session-park`

See `plugins/README.md` and designs: `docs/CHEF-PHASE5-DESIGN.md`, `docs/CHEF-SETTINGS-UI-DESIGN.md`, `docs/CHEF-KATER-BRIDGE-DESIGN.md`.

## Fleet Ops Bar + Settings

- Toggle: Settings → **fleet** → `ui.fleet_ops_bar` (hot-reload)
- Catalog browse: Settings → **plugins**
- Render: `src/ui/panes.rs` `render_fleet_ops_bar` (gated on `fleet_ops_bar_enabled()`)
- Personal summary: `FleetOpsMetadata::personal_summary_line()`

## Pi-Memory (Phase 6)

Pi integration calls `pi-memory` on lifecycle (`src/integration/assets/pi/herdr-agent-state.ts`). Resume on Unix runs sync/state/query before `pi --session`. Disable with `HERDR_PI_MEMORY=0`.

## What we do NOT build

GPU/Sixel, new plugin framework, screenshots-as-core, Combined stale SSOT — see ADR.

## Validation

Prefer `just check`. Cursor Cloud: GitHub Actions on the PR.
