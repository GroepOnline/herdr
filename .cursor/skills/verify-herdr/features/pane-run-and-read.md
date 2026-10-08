# Pane run + read

## What

A user creates a workspace, runs a shell command in a pane, and reads the pane output.

## Reach

1. Isolated headless server running (`scripts/launch-server.sh`).
2. `workspace create --cwd <home> --focus` so a pane exists.
3. Discover pane id: `pane list` (fresh default is usually `w1:p1`).

## Drive

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/launch-server.sh
.cursor/skills/verify-herdr/scripts/doctor.sh
create_json="$(.cursor/skills/verify-herdr/scripts/cli.sh workspace create --cwd "$HERDR_HOME" --focus)"
PANE="$(printf '%s' "$create_json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["root_pane"]["pane_id"])')"
.cursor/skills/verify-herdr/scripts/cli.sh pane run "$PANE" echo 'verify-herdr-ok'
.cursor/skills/verify-herdr/scripts/cli.sh pane wait-output "$PANE" --match 'verify-herdr-ok' --timeout 5000
.cursor/skills/verify-herdr/scripts/cli.sh pane read "$PANE" --source recent --format text
```

Or one shot: `.cursor/skills/verify-herdr/scripts/smoke-pane-run-read.sh`

## Prove

- `pane wait-output` exits 0 for the marker.
- `pane-read.txt` (or stdout) contains the marker string.
- Evidence under `$HERDR_VERIFY_EVIDENCE` still present after `cleanup.sh`.
