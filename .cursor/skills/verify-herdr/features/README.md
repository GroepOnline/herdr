# Herdr verification feature map

User-facing features to prove with `.cursor/skills/verify-herdr/`. Drive the **real** headless server + CLI (or TUI) path named in each file. Prefer these entry points over one-off shortcuts.

| Feature | File | Surface |
| --- | --- | --- |
| Headless server lifecycle | `server-lifecycle.md` | CLI `server` / `status` |
| Workspace create + focus | `workspace-create-focus.md` | CLI `workspace` |
| Pane run + read | `pane-run-and-read.md` | CLI `pane` (primary smoke) |
| Pane send-text / wait-output | `pane-send-and-wait.md` | CLI `pane` |
| Status JSON doctor | `status-json.md` | CLI `status --json` |

Start here for a cold proof: `pane-run-and-read.md` via `scripts/smoke-pane-run-read.sh`.
