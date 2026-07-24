# Phase 5 — CHEF Plugins Design

6 plugins built on Herdr's existing plugin registry (v0.7.3).

## Plugin Manifest Schema

Herdr plugins use `herdr-plugin.toml` with: `id`, `name`, `version`, `min_herdr_version`, `[[build]]`, `[[actions]]`, `[[events]]`, `[[panes]]`, `[[link_handlers]]`.

21 hookable events available. 32 max concurrent plugin commands. State dir: `~/.local/state/herdr/plugins/<id>/`.

## Plugins

### 1. Linear Issue Context (`com.chefgroep.linear-context`)
- **SSOT:** Linear GraphQL API
- **Actions:** fetch-issue, fetch-cycle, set-issue
- **Events:** worktree.created (auto-extract issue from branch), workspace.focused
- **TTL:** 60s | **Poll:** 60s background

### 2. GitHub PR/CI Status (`com.chefgroep.github-status`)
- **SSOT:** GitHub REST + GraphQL API
- **Actions:** check-pr, list-open-prs, watch-checks
- **Events:** worktree.opened
- **TTL:** 30s | **Poll:** 15s CI watch, 30s bar refresh

### 3. Issue-to-Workspace Provisioning (`com.chefgroep.issue-provision`)
- **SSOT:** Linear + Git
- **Actions:** provision, teardown, list-provisioned
- **Link handler:** Linear issue URL → provision workspace
- One-shot, not a poller

### 4. Fleet Health (`com.chefgroep.fleet-health`)
- **SSOT:** Tailscale API + SSH probes
- **Actions:** scan-fleet, scan-node
- **Events:** workspace.focused (throttled)
- **TTL:** 120s | SSH timeout: 5s/node

### 5. Cloudflare Tunnel Status (`com.chefgroep.cloudflare-tunnel`)
- **SSOT:** Cloudflare API
- **Actions:** check-tunnels, check-dns, dig-probe
- **Events:** workspace.focused (throttled)
- **TTL:** 300s

### 6. Session Parking (`com.chefgroep.session-park`)
- **SSOT:** Herdr session snapshot system
- **Actions:** park, park-workspace, resume, list-parked, expire
- **Events:** pane.agent_status_changed, pane.exited
- Persistent records, not cached

## Fleet Ops Bar (Aggregator)

**Core host (herdr):** refreshes a `FleetOpsCache` on a timer from *installed*
plugin ids only (no hardcoded `com.chefgroep.*` allowlist, no sync disk I/O in
`render()`). Issue labels that already look like `KEY-123` (e.g. `ENG-432`)
are shown as-is; bare numeric ids get a neutral `LIN-` prefix. Plugins write
personalized keys into `issue.id`.

**Plugin home:** CHEF scaffolds + Linear/GitHub implementations live in
`OnlineChefGroep/herdr-plugins` (make that repo private for personalization).
Do not land kitchen-sink Fleet/Plugins settings tabs or Utrecht/Kater copy in
core.

Reads `fleet_ops.json` from installed plugin state dirs, merges into unified view:
```
ENG-432 ● In Progress · joep · Sprint W28 │ PR #42 ✓ CI: passing
Fleet: 2/3 online (jan↓) · RAM avg 44%    │ CF: 2 tunnels healthy
Parked: 3 sessions (oldest 48h)            │ Updated: 30s ago
```

## State-Dir Contract

```
$HERDR_PLUGIN_STATE_DIR/
  fleet_ops.json    # metadata fragment (TTL-based)
  cache/            # API response cache
  parked/           # session park records (plugin 6)
```

## Security

- No secrets in `fleet_ops.json`, stdout, or logs
- API keys in `$HERDR_PLUGIN_CONFIG_DIR/.env` (0600)
- SSH: BatchMode, ControlMaster, existing key auth
- Cloudflare: scoped API tokens (not global key)

## Implementation Order

- Week 1: Plugin 1 (Linear) + Plugin 2 (GitHub) + Fleet Ops Bar
- Week 2: Plugin 4 (Fleet Health) + Plugin 5 (Cloudflare)
- Week 3: Plugin 3 (Issue Provision) + Plugin 6 (Session Parking)

