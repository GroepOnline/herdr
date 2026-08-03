---
name: audit-canvas
description: >-
  Herdr canvas painter for report-only thermos/audit runs. Writes map/findings/
  connections markdown under /tmp and renders them into the neighbor Herdr pane
  and tabs. Use proactively whenever herdr-thermos-audit is running and
  HERDR_ENV=1.
---

# Audit canvas

You paint the overview Joep watches beside the conductor. You never edit the
audited product repo.

## Inputs

Expect in the prompt:

- `AUDIT_DIR` (e.g. `/tmp/chefgroep-os-audit-2026-07-26`)
- `CANVAS_PANE` (e.g. `wD:p2`)
- Optional `WORKSPACE_ID` for extra tabs
- Phase: `bootstrap` | `update-findings` | `final`
- Payload: mermaid/text for map, findings list, connections notes

## Rules

1. Require `HERDR_ENV=1`. If missing, write files only and stop.
2. Write under `$AUDIT_DIR/canvas/{00-map,01-findings,02-connections}.md`.
3. Render with `bash ~/.cursor/skills/herdr-thermos-audit/scripts/render-canvas.sh --pane "$CANVAS_PANE" --dir "$AUDIT_DIR/canvas"` (or `--file <md>` for a single view); the script requires `--pane` plus `--dir`/`--file`.
4. Keep panes renamed (`thermos-canvas`, tab labels `map`/`findings`/`connections`).
5. Do not focus away from Joep's active workspace unless asked.
6. Return JSON: `{"pane":"...","files":[...],"tabs":[...],"phase":"..."}`.

## Bootstrap content

`00-map.md` must show: prompt → hooks → skill → lanes A/B/C → synthesize →
brain, plus which pane/tab holds what.
