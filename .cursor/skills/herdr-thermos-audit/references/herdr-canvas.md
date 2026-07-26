# Herdr canvas for thermos audits

Requires `HERDR_ENV=1`. Conductor stays in its pane; **neighbor pane** + **extra tabs** are the canvas.

## Discover

```bash
test "${HERDR_ENV:-}" = 1 || { echo "not in herdr"; exit 1; }
herdr pane list
herdr tab list
herdr workspace list
```

Prefer the other pane on the conductor tab for `WHERE-LIVE.md`. Create dedicated tabs for map / findings / connections.

## Bootstrap

```bash
AUDIT=/tmp/<repo>-audit-<date>/canvas
mkdir -p "$AUDIT"

# Neighbor on conductor tab (adjust pane id from pane list)
herdr pane split <conductor-pane> --direction right --ratio 0.45 --cwd "$AUDIT" --no-focus
herdr pane rename <new-pane> where-live

herdr tab create --workspace <ws> --cwd "$AUDIT" --label map --no-focus
herdr tab create --workspace <ws> --cwd "$AUDIT" --label findings --no-focus
herdr tab create --workspace <ws> --cwd "$AUDIT" --label connections --no-focus
```

Write `WHERE-LIVE.md`, `00-map.md`, `01-findings.md`, `02-connections.md`, later `03-FINAL.md`.

## Render (sticky, scrollable)

Always use the less binary directly — no `bash -lc`, no preceding `send-keys q`:

```bash
herdr pane run <pane> /usr/bin/less -R "$AUDIT/WHERE-LIVE.md"
herdr pane run <map-pane> /usr/bin/less -R "$AUDIT/00-map.md"
herdr pane run <findings-pane> /usr/bin/less -R "$AUDIT/01-findings.md"
herdr pane run <connections-pane> /usr/bin/less -R "$AUDIT/02-connections.md"
```

Or: `bash ~/.cursor/skills/herdr-thermos-audit/scripts/render-canvas.sh --pane <id> --file <md>`

Reload after edits: `herdr pane send-keys <pane> R` (less reload) or re-run less.

Focus Joep on findings when fan-in lands:

```bash
herdr workspace focus <ws>
herdr tab focus <findings-tab-id>
```

## Anti-patterns

- `glow` / `less` via `bash -lc` after `send-keys q` → `qbash` paste bugs
- Closing overview tabs accidentally when re-splitting the conductor tab
- Stealing focus from Joep's other workspace without need
