# Phase 3 — Crash-Safe Recovery Design

## systemd User Service

```ini
# ~/.config/systemd/user/herdr.service
[Unit]
Description=Herdr Terminal Multiplexer Server (CHEF Fleet)
After=network.target

[Service]
Type=simple
ExecStart=%h/.local/bin/herdr.real server
ExecReload=kill -HUP $MAINPID
Restart=on-failure
RestartSec=2
Environment=HERDR_RENDER_ENCODING=ansi
Environment=HERDR_UPDATE_BASE_URL=https://github.com/GroepOnline/herdr/releases/download

[Install]
WantedBy=default.target
```

Enable: `systemctl --user enable --now herdr`

## Client Auto-Reconnect

```
 ┌──────────┐     ┌──────────┐
 │  Client  │────▶│  Server  │
 │  (TUI)   │◀────│  (sock)  │
 └──────────┘     └──────────┘
       │
       │  connection lost
       ▼
 ┌──────────┐
 │ Reconnect│  backoff: 1s → 2s → 4s → 8s → 16s (cap)
 │ Loop     │  max retries: 10
 └──────────┘
       │
       │  reconnected
       ▼
 ┌──────────┐
 │ Restore  │  1. Request session snapshot
 │ State    │  2. Verify panes, cwd, worktree
 │          │  3. Resume agent if session_resume_available
 └──────────┘
```

## Heartbeat Protocol

| Direction | Message | Interval |
|---|---|---|
| Server → Client | `heartbeat` (seq, timestamp) | 5s |
| Client → Server | `pong` (echoed seq) | on receipt |
| Detection | No pong within 15s → stale | Server kills connection |
| Detection | No heartbeat within 15s → server down | Client enters reconnect |

## Atomic Snapshots

Snapshot write path:
1. Write to `~/.local/share/herdr/sessions/<id>.tmp`
2. `fsync()` the file
3. `rename()` to `~/.local/share/herdr/sessions/<id>.snap`
4. `fsync()` the directory

This is crash-safe: either the old snapshot exists or the new one.

## Pane Restore Policies

| Policy | Behavior | When |
|---|---|---|
| `shell-only` | Fresh shell in saved cwd | Default, no agent |
| `resume-agent` | Restore agent session from persisted state | session_resume_available=true |
| `restart-command` | Re-run the original launch argv | launch_argv present |
| `manual` | No auto-restore, user picks | Config: chef.restore_policy=manual |

## Restore Verification

After restore, verify:
1. Process is alive (`kill(pid, 0)`)
2. cwd matches snapshot
3. Worktree branch matches snapshot
4. Session-id matches (if agent)
5. Integration state (opencode plugin, etc.)

If any check fails → `RecoveryFailed` state + audit log entry.

## Recovery Audit Log

```
~/.local/share/herdr/recovery.log (JSONL)
{"ts":"2026-07-12T06:00:00Z","event":"reconnect","session":"abc","delay_ms":2000,"result":"ok"}
{"ts":"2026-07-12T06:00:05Z","event":"restore","pane":3,"policy":"resume-agent","result":"ok","checks":{"cwd":true,"branch":true,"pid":true}}
{"ts":"2026-07-12T06:00:10Z","event":"restore","pane":5,"policy":"shell-only","result":"failed","reason":"cwd_missing","checks":{"cwd":false}}
```

## RecoveryFailed State

New AgentState variant (or metadata flag):
- `AgentState::Unknown` + `recovery_failed: true` in FleetOpsMetadata
- Pane border rendered in red
- Toast notification: "Pane 3 recovery failed: cwd missing"
- User can manually retry or pick a restore policy

## Explicit Non-Claims

- Agent processes do NOT survive a reboot
- Agent conversation history may be lost
- Only structural state (panes, cwd, worktree, metadata) is restored
- Agent resume is best-effort via persisted_agent_session

