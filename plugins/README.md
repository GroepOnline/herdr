# CHEF plugins

Local scaffolds for the CHEF Fleet Operations plugins. Each directory is a
self-contained Herdr plugin (`herdr-plugin.toml` + Node ESM `src/index.js`).

## Install from path

Herdr registers local checkouts with `plugin link` (GitHub shorthand uses
`plugin install`). From a Herdr-capable machine:

```bash
herdr plugin link /workspace/plugins/linear-context
herdr plugin link /workspace/plugins/github-status
herdr plugin link /workspace/plugins/kater-bridge
herdr plugin link /workspace/plugins/udo-metrics
herdr plugin link /workspace/plugins/fleet-health
herdr plugin link /workspace/plugins/cloudflare-tunnel
herdr plugin link /workspace/plugins/issue-provision
herdr plugin link /workspace/plugins/session-park
```

Published copies will later install as GitHub shorthand, for example:

```bash
herdr plugin install OnlineChefGroep/herdr-plugins/chef-linear-context
```

Requires Herdr `>= 0.7.5` (`min_herdr_version` in each manifest).

## State-dir contract

Herdr injects `HERDR_PLUGIN_STATE_DIR` and `HERDR_PLUGIN_CONFIG_DIR` on every
plugin command. Plugins own the files under those paths.

```text
$HERDR_PLUGIN_STATE_DIR/
  fleet_ops.json    # metadata fragment for the Fleet Ops Bar aggregator
  cache/            # optional API response cache (plugin-owned)
  parked/           # session-park records only
```

`fleet_ops.json` shape (fill only fields this plugin owns; omit or null the rest):

```json
{
  "source": "linear-context",
  "updated_at": "<iso>",
  "ttl_seconds": 60,
  "issue": {"id":"", "title":"", "status":"", "assignee":"", "cycle":""},
  "pr": {"number":null, "checks":""},
  "fleet": {"online":null, "total":null, "summary":""},
  "cloudflare": {"tunnels_healthy":null, "summary":""},
  "parked": {"count":null, "oldest_hours":null}
}
```

Rules:

- Treat `ttl_seconds` as advisory freshness for the aggregator. Do not write
  secrets into `fleet_ops.json`, stdout, or logs.
- Put API keys in `$HERDR_PLUGIN_CONFIG_DIR/.env` (mode `0600`). Example keys:
  `LINEAR_API_KEY`, `GITHUB_TOKEN`, Cloudflare tokens.
- `session-park` writes durable records under `parked/` and mirrors a count /
  oldest-age summary into `fleet_ops.json`.

## SSOT rules

| Concern | Source of truth | Plugin(s) |
|---|---|---|
| Issue / assignee / cycle | Linear GraphQL | `linear-context`, `issue-provision` |
| PR / CI checks | GitHub REST + GraphQL | `github-status` |
| Host / node online | Tailscale + SSH probes | `fleet-health` |
| Utrecht fleet inventory | Kater MCP / Utrecht Data OS | `kater-bridge`, `udo-metrics` |
| Tunnel / DNS health | Cloudflare API | `cloudflare-tunnel` |
| Parked sessions | Herdr session snapshots + `parked/` | `session-park` |

Do not invent a parallel issue or PR database inside Herdr state. Plugins
project remote SSOT into short-lived `fleet_ops.json` fragments for the bar.

## Plugins

| Directory | Id |
|---|---|
| `linear-context/` | `com.chefgroep.linear-context` |
| `github-status/` | `com.chefgroep.github-status` |
| `kater-bridge/` | `com.chefgroep.kater-bridge` |
| `udo-metrics/` | `com.chefgroep.udo-metrics` |
| `fleet-health/` | `com.chefgroep.fleet-health` |
| `cloudflare-tunnel/` | `com.chefgroep.cloudflare-tunnel` |
| `issue-provision/` | `com.chefgroep.issue-provision` |
| `session-park/` | `com.chefgroep.session-park` |

Run an action headlessly:

```bash
export HERDR_PLUGIN_STATE_DIR=/tmp/chef-plugin-state/linear-context
export HERDR_PLUGIN_CONFIG_DIR=/tmp/chef-plugin-config/linear-context
node /workspace/plugins/linear-context/src/index.js fetch-issue
```
