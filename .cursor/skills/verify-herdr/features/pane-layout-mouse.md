# Pane layout (mouse + CLI)

## What

Create extra panes and prove focus/layout — mouse context menus in the TUI, CLI `pane split` / `pane list` for headless proofs.

## Reach

- **TUI:** real TTY / desktop; isolated `HOME`
- **CLI:** headless server via `launch-server.sh`

## Drive — mouse (TUI)

1. Right-click inside a pane → **Split right** / **Split down**.
2. Click panes to change focus (active pane border changes).
3. Optional: resize via drag handles if the build exposes them.
4. Run a marker command in each focused pane (`echo mouse-pane-N`).

## Drive — CLI

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/launch-server.sh
create_json="$(.cursor/skills/verify-herdr/scripts/cli.sh workspace create --cwd "$HERDR_HOME" --focus)"
PANE="$(printf '%s' "$create_json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["root_pane"]["pane_id"])')"
.cursor/skills/verify-herdr/scripts/cli.sh pane split "$PANE" --direction right --focus
.cursor/skills/verify-herdr/scripts/cli.sh pane split "$PANE" --direction down --focus
.cursor/skills/verify-herdr/scripts/cli.sh pane list
```

Covered in `scripts/e2e.sh` (single split). Extend locally for multi-split layouts when proving layout templates.

## Prove

- `pane list` shows ≥2 panes after split (CLI), or screenshot shows ≥2 panes (TUI).
- Focused pane accepts `pane run` / typed input.
- Evidence: `pane-list-after-split.txt` or mouse screenshots under `/opt/cursor/artifacts/screenshots/herdr-mouse-*.webp`.
