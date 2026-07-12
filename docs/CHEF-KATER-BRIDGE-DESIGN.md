# Kater MCP Bridge — Herdr Integration Design

## Kater MCP Gateway (sofie)

**Endpoint:** `http://127.0.0.1:9091` (API) + `:9090/sse` (MCP SSE) + `:9092` (WS)
**Profiles:** utrecht, ops, cloud, research, code, web
**Auth:** none (localhost only)

### Utrecht Tools Available

| Tool | Purpose |
|---|---|
| `utrecht_status` | Utrecht Data OS pipeline status |
| `utrecht_pipeline_status` | Data pipeline health |
| `utrecht_fleet_inventory` | Fleet node inventory |
| `utrecht_agent_manifest` | Agent detection manifests |
| `kater_profiles` | List available tool profiles |
| `kater_doctor` | Diagnostics |
| `kater_chains` | Tool chains |
| `kater_adapters` | External MCP adapter scan |
| `kater_config` | Full MCP config render |

## Herdr Integration

### Plugin: `com.chefgroep.kater-bridge`

```toml
id = "com.chefgroep.kater-bridge"
name = "Kater MCP Bridge"
version = "0.1.0"
min_herdr_version = "0.7.3"
description = "Bridge to Kater MCP Gateway on sofie for Utrecht Data OS integration"
platforms = ["linux"]

[[actions]]
id = "fleet-status"
title = "Fleet Status"
description = "Get live fleet inventory from Utrecht Data OS"
contexts = ["global", "workspace"]
command = ["node", "dist/fleet-status.js"]

[[actions]]
id = "pipeline-health"
title = "Pipeline Health"
description = "Check data pipeline status"
contexts = ["global"]
command = ["node", "dist/pipeline.js"]

[[actions]]
id = "query"
title = "Query Utrecht Data"
description = "Natural language query via Kater"
contexts = ["pane"]
command = ["node", "dist/query.js"]

[[panes]]
id = "fleet-dashboard"
title = "Utrecht Fleet"
placement = "tab"
command = ["node", "dist/dashboard.js"]

[[events]]
on = "workspace.focused"
command = ["node", "dist/refresh.js"]
```

### Data Flow

```
Herdr Fleet Ops Bar
    ↓ reads fleet_ops.json
Kater Bridge Plugin
    ↓ HTTP GET :9091/api/tools/utrecht_fleet_inventory
Kater MCP Gateway (sofie:9091)
    ↓ reads fleet.json
Utrecht Fleet Inventory (/home/sofie/utrecht-fleet/inventory/fleet.json)
    ↓ SSH probes (cron /15min)
11 Tailscale nodes
```

### Unified Health Aggregator

Merge data from:
1. **Kater MCP** → utrecht_fleet_inventory (node status, roles)
2. **Cloudflare API** → tunnel health (5 tunnels)
3. **Tailscale API** → device list, last_seen
4. **SSH probes** → CPU/RAM/disk per node
5. **Grafana API** (bc-scan-arm:3000) → historical metrics

Single endpoint: `GET /v1/fleet/health` on Herdr Gateway

### Cloudflare Tunnel Correlation

| Tunnel | CF Status | Target Node | Correlation |
|---|---|---|---|
| mc-api | ✅ Healthy | sofie | Kater cloudflared |
| pi-helios-memory-hub | ✅ Healthy | bc-scan-arm | Workers |
| agent-platform-tunnel | ❌ Down | ? | Investigate |
| helios-hub-tunnel | ❌ Inactive | ? | Investigate |
| openclaw-admin | ❌ Down | ? | Investigate |

Bridge checks: CF tunnel down → SSH target node → root cause → alert in Fleet Ops Bar.

