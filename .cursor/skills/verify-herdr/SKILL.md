---
name: verify-herdr
description: "Drive the Herdr terminal agent runtime (headless server + socket CLI, optional TUI) the way a user would. Use when proving pane/workspace/CLI behavior, smoke-testing a binary, or capturing transcripts without cargo builds on Cloud VMs."
---

# Verify Herdr

Herdr is a terminal workspace manager: a **headless server** owns panes/PTY state; the **CLI** talks over a Unix socket; the **TUI** is one client. Primary verification surface here is **headless server + CLI**. Interactive TUI is optional (needs a real TTY / desktop).

On Cursor Cloud VMs, local `cargo` / `just check|test|lint` are blocked. Use a CI smoke binary (see Launch). Never drive the user's installed stable server — always isolate with a dedicated `HOME`.

## Launch

### Binary

Prefer an explicit binary:

```bash
export HERDR_BIN=/tmp/herdr-bin-fresh/herdr-linux-x86_64
```

If missing, download from a green CI run (artifact `ci-smoke-herdr-linux-x86_64`):

```bash
gh run list -R OnlineChefGroep/herdr --workflow=ci.yml --limit 5 \
  --json databaseId,conclusion,headBranch
# pick a success run-id that includes release-build / smoke artifact
gh run download <run-id> -R OnlineChefGroep/herdr \
  -n ci-smoke-herdr-linux-x86_64 -D /tmp/herdr-bin-fresh
chmod +x /tmp/herdr-bin-fresh/herdr-linux-x86_64
export HERDR_BIN=/tmp/herdr-bin-fresh/herdr-linux-x86_64
```

Helper (resolves `HERDR_BIN`, creates isolated home, prints env):

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
```

### Start isolated headless server

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/launch-server.sh
```

Ready when `$HERDR_HOME/.config/herdr/herdr.sock` accepts connects and
`HOME=$HERDR_HOME "$HERDR_BIN" status --json` shows `"running": true`.

Teardown: `.cursor/skills/verify-herdr/scripts/cleanup.sh` (kills the PID recorded at launch; does not delete evidence).

### Optional interactive TUI

Needs a real TTY (Cloud: xfce4 on `:1`). Symlink the smoke binary and run `herdr` in a desktop terminal, or use the tmux harness in **Drive → TUI**. Do not attach a verification TUI to a non-isolated `HOME`.

## Doctor

Read-only health check for the isolated instance:

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/doctor.sh
```

Pass criteria:

- `HERDR_BIN` exists and `--version` prints `herdr …`
- socket path under `$HERDR_HOME/.config/herdr/herdr.sock`
- `status --json` → `server.running == true`, `server.compatible != false`
- recorded server PID is still alive

If doctor fails, stop and relaunch — do not drive a half-dead shared instance.

## Drive

Always wrap CLI calls:

```bash
herdr_cli() { HOME="$HERDR_HOME" env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH "$HERDR_BIN" "$@"; }
```

Or use the helper: `.cursor/skills/verify-herdr/scripts/cli.sh <args…>`

### Headless CLI recipe (preferred)

Stable handles are pane/workspace IDs from `workspace list` / `pane list` (often `w1:p1` after first create), plus JSON from `status --json` / `pane read … --format text`.

Minimal happy path (also scripted in `scripts/smoke-pane-run-read.sh`):

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/launch-server.sh
.cursor/skills/verify-herdr/scripts/doctor.sh
create_json="$(.cursor/skills/verify-herdr/scripts/cli.sh workspace create --cwd "$HERDR_HOME" --focus)"
PANE="$(printf '%s' "$create_json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["root_pane"]["pane_id"])')"
.cursor/skills/verify-herdr/scripts/cli.sh pane list
.cursor/skills/verify-herdr/scripts/cli.sh pane run "$PANE" echo 'verify-herdr-ok'
.cursor/skills/verify-herdr/scripts/cli.sh pane wait-output "$PANE" --match 'verify-herdr-ok' --timeout 5000
.cursor/skills/verify-herdr/scripts/cli.sh pane read "$PANE" --source recent --format text
```

Send keys / text when proving input paths (check `pane send-text --help` / `pane send-keys --help` for exact flags on the binary under test):

```bash
.cursor/skills/verify-herdr/scripts/cli.sh pane send-text "$PANE" 'echo hi'
.cursor/skills/verify-herdr/scripts/cli.sh pane send-keys "$PANE" Enter
```

### TUI tmux harness (optional)

```bash
SESSION="herdr-cli-verify-$$"
tmux -f /exec-daemon/tmux.portal.conf new-session -d -s "$SESSION" -c "$HERDR_HOME" -- bash -lc \
  'env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH HOME="'"$HERDR_HOME"'" "'"$HERDR_BIN"'" status --json; sleep 2'
sleep 1
tmux -f /exec-daemon/tmux.portal.conf capture-pane -pt "$SESSION" | tee "$HERDR_VERIFY_EVIDENCE/tmux-status-capture.txt"
tmux -f /exec-daemon/tmux.portal.conf kill-session -t "$SESSION"
```

For a long-lived interactive TUI, omit `status --json; sleep` and run `"$HERDR_BIN"` alone (needs a real TTY/desktop). Prefix is ctrl+b by default — prefer CLI for functional proofs.

Prefer CLI proofs for pane content; use TUI only when verifying layout/keybind UX.

Feature-specific steps live under `features/`. Prefer those entry points over inventing a one-off path.

## Evidence

Proof directory (survives cleanup):

```text
/opt/cursor/artifacts/herdr-verify/<run-id>/
```

`scripts/env.sh` sets `HERDR_VERIFY_EVIDENCE` to a fresh run dir. Capture:

| Artifact | How |
| --- | --- |
| `doctor.json` | `status --json` at doctor time |
| `pane-list.json` / text | `pane list` (+ `--json` when available) |
| `pane-read.txt` | `pane read <id> --source recent --format text` after the action |
| `cli-transcript.log` | stdout/stderr of the drive script |
| `server.log` | copy of `$HERDR_HOME/.config/herdr/herdr-server.log` if present |

Proof standards:

- Exercise the real CLI/user path (`workspace create`, `pane run`, `pane read`), not internal setters.
- Capture **action + resulting pane text**, not only exit codes.
- Side effects: pane output contains the expected marker; workspace appears in `workspace list`.
- No mocks for the PTY path — the smoke binary runs real panes.

## Cleanup

```bash
.cursor/skills/verify-herdr/scripts/cleanup.sh
```

- Stops the server via `herdr server stop` when the socket still answers, else kills the **recorded PID** only.
- Removes `$HERDR_HOME` scratch tree.
- **Never** deletes `$HERDR_VERIFY_EVIDENCE`.
- Does not `pkill herdr` / kill by process name.

## Helpers

All under `.cursor/skills/verify-herdr/scripts/` (executable):

| Script | Role |
| --- | --- |
| `env.sh` | Resolve binary, allocate `HERDR_HOME` + evidence dir, export vars |
| `launch-server.sh` | Start headless server; write PID/socket metadata |
| `doctor.sh` | Read-only instance health |
| `cli.sh` | Isolated CLI wrapper |
| `smoke-pane-run-read.sh` | End-to-end proof of `pane-run-and-read` feature |
| `cleanup.sh` | Tear down instance; keep evidence |

## Feature map

See `features/README.md`. Start with `pane-run-and-read` for a single proof run.
