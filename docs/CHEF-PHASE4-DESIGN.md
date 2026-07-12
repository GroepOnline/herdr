# Phase 4 — CHEF API Gateway Design

localhost-bound HTTP gateway on top of the Herdr socket API.

## Endpoints

### GET /health
```json
{
  "status": "ok",
  "version": "0.7.3-chef",
  "uptime": 3600,
  "sessions": 2,
  "panes": 8
}
```

### GET /v1/workspaces
```json
[
  {"id": 0, "name": "main", "tabs": 2, "panes": 4},
  {"id": 1, "name": "fleet", "tabs": 1, "panes": 2}
]
```

### GET /v1/session
```json
{
  "id": "abc123",
  "created": "2026-07-12T05:00:00Z",
  "workspaces": [...],
  "snapshots": ["snap-1", "snap-2"]
}
```

### GET /v1/agents
```json
[
  {
    "pane_id": 3,
    "agent": "claude",
    "state": "working",
    "label": "chef-bot",
    "repo": "herdr",
    "branch": "master",
    "model": "glm-5.2",
    "host": "sofie",
    "elapsed": 300,
    "retry_count": 0,
    "session_resume_available": true
  }
]
```

### GET /v1/agents/:id
Single agent detail.

### GET /v1/events (SSE)
```
data: {"type":"state_change","pane":3,"from":"working","to":"blocked"}

data: {"type":"recovery","pane":5,"result":"ok","policy":"resume-agent"}

data: {"type":"ci_status","pr":42,"status":"success"}
```

## Security

- Bind to `127.0.0.1` only
- Read-only by default
- Mutation endpoints require `Authorization: Bearer <token>`
- Token in `~/.config/herdr/gateway-token`
- Audit log at `~/.local/share/herdr/gateway-audit.log`

## Implementation

Built as a separate `herdr-gateway` binary that reads from the socket
and serves HTTP. Does not modify the core Herdr server.

```
┌────────────┐     ┌─────────────────┐     ┌──────────┐
│  Browser   │────▶│  herdr-gateway   │────▶│  Herdr   │
│  / curl    │ HTTP│  (localhost:7777)│ IPC │  Server  │
└────────────┘     └─────────────────┘     └──────────┘
```

