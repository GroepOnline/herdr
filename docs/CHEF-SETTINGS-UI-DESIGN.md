# CHEF Settings UI + Plugin Marketplace Design

## Settings Overlay (Prefix + S)

Accessible via `Ctrl+A, S` — modal overlay with tabbed sections.

### Layout

```
┌─ Settings ──────────────────────────────────────────────┐
│ ▸ General  ◸ Appearance  ◸ Agents  ◸ Fleet  ◸ Plugins  │
│                                                         │
│  Prefix key:        [Ctrl+A ▾]                         │
│  Render encoding:   [TerminalAnsi ▾]                    │
│  Focus redraw:      [○ Off]                             │
│  Border labels:     [● On]                              │
│  Theme:             [Catppuccin Mocha ▾]                │
│  Default shell:     [/usr/bin/fish ▾]                   │
│  Sound:             [○ Off]                             │
│  Toast delivery:    [Terminal ▾]                        │
│                                                         │
│  [Save]  [Cancel]  [Reset to defaults]                 │
└─────────────────────────────────────────────────────────┘
```

### Implementation

New file: `src/ui/settings.rs`
- Modal overlay using existing `render_pane_border_titles` pattern
- Reads/writes `~/.config/herdr/config.toml`
- Hot-reloads on save (no restart needed for most settings)
- Tab navigation: left/right arrows
- Toggle: enter/space

### Tabs

| Tab | Options |
|---|---|
| General | prefix key, default shell, update channel, version check |
| Appearance | theme, antialiasing, scrollbar, border style |
| Agents | border labels, agent sounds, detection manifests list |
| Fleet | fleet ops bar toggle, fleet ops bar position, host label |
| Plugins | installed plugins, enable/disable, marketplace browser |

## Plugin Marketplace (Settings → Plugins tab)

```
┌─ Plugin Marketplace ────────────────────────────────────┐
│ [Installed]  [Browse]  [Search...]                      │
│                                                         │
│ ● com.chefgroep.linear-context     v0.1.0  [Enabled]    │
│   Linear issue context in Fleet Ops Bar                 │
│                                                         │
│ ○ com.chefgroep.github-status      v0.1.0  [Install]    │
│   GitHub PR/CI status polling                           │
│                                                         │
│ ○ com.chefgroep.fleet-health       v0.1.0  [Install]    │
│   Tailscale node health + SSH probes                    │
│                                                         │
│ ○ com.chefgroep.cloudflare-tunnel  v0.1.0  [Install]    │
│   Cloudflare tunnel + DNS health                        │
│                                                         │
│ ○ com.chefgroep.session-park       v0.1.0  [Install]    │
│   Snapshot and resume agent sessions                    │
│                                                         │
│ ○ com.chefgroep.issue-provision    v0.1.0  [Install]    │
│   Create workspace from Linear issue                    │
│                                                         │
│─────────────────────────────────────────────────────── │
│ Source: github.com/OnlineChefGroep/herdr-plugins       │
│ [Refresh]  [Install All]  [Update All]                  │
└─────────────────────────────────────────────────────────┘
```

### Marketplace Registry

`~/.config/herdr/marketplace.toml`:
```toml
[source]
url = "https://github.com/OnlineChefGroep/herdr-plugins"
branch = "main"
registry_path = "registry.json"

[cache]
ttl_seconds = 3600
dir = "~/.cache/herdr/marketplace"
```

`registry.json` (in the plugins repo):
```json
{
  "plugins": [
    {
      "id": "com.chefgroep.linear-context",
      "name": "Linear Issue Context",
      "version": "0.1.0",
      "description": "Linear issue context in Fleet Ops Bar",
      "homepage": "https://github.com/OnlineChefGroep/herdr-plugins/tree/main/chef-linear-context",
      "categories": ["productivity", "linear"],
      "min_herdr_version": "0.7.3"
    }
  ]
}
```

### Plugin Operations

| Action | CLI | UI |
|---|---|---|
| Install | `herdr plugin install <id>` | [Install] button |
| Enable | `herdr plugin enable <id>` | Toggle ●/○ |
| Disable | `herdr plugin disable <id>` | Toggle ●/○ |
| Uninstall | `herdr plugin uninstall <id>` | [Uninstall] button |
| Update | `herdr plugin update <id>` | [Update] button |
| Configure | `herdr plugin config <id>` | [Configure] → .env editor |

### Utrecht Data OS Integration

The marketplace also lists fleet integration plugins:

| Plugin | Purpose |
|---|---|
| `com.chefgroep.fleet-health` | Tailscale + SSH fleet monitoring |
| `com.chefgroep.kater-bridge` | Bridge to Kater MCP on sofie |
| `com.chefgroep.udo-metrics` | Pull metrics from Utrecht Data OS Grafana/Netdata |
| `com.chefgroep.cloudflare-tunnel` | Cloudflare tunnel + DNS status |

These plugins read from existing fleet infrastructure (MCP servers, Grafana, Netdata) without creating new sources of truth.

## Workspace Templates

`~/.config/herdr/templates/` with TOML files:

```toml
# dev.toml — standard dev workspace
name = "Development"
[[tab]]
name = "Editor"
[[tab.pane]]
command = "nvim"
cwd = "${REPO_ROOT}"
[[tab.pane]]
direction = "right"
command = "fish"

[[tab]]
name = "Fleet"
[[tab.pane]]
command = "herdr fleet-dashboard"
```

```toml
# incident.toml — incident response
name = "Incident Response"
[[tab]]
name = "Monitor"
[[tab.pane]]
command = "ssh sofie-lan htop"
[[tab.pane]]
direction = "right"
command = "ssh bc-scan-arm kubectl get pods -A"
```

Command: `herdr new --template dev` or `herdr new --template incident`

