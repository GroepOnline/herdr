# Herdr canvas for thermos audits

Requires `HERDR_ENV=1`. Conductor stays in its pane; **neighbor pane** + **extra tabs** are the canvas.

## 0. Always look first (mandatory)

Before creating anything, inventory what already exists:

```bash
test "${HERDR_ENV:-}" = 1 || { echo "not in herdr"; exit 1; }
herdr workspace list
herdr tab list
herdr pane list
herdr worktree list
ls -la /tmp/<repo>-audit-*/canvas 2>/dev/null || true
```

**Reuse rules**

| Found | Action |
|-------|--------|
| Tabs labeled `map` / `findings` / `connections` | Reuse those tab/pane ids — do **not** create duplicates |
| Neighbor pane labeled `where-live` | Reuse; only re-render content |
| Canvas dir missing under `/tmp` but tabs exist | Recreate markdown from joep-brain reports, then `herdr pane run … /usr/bin/less -R` on **existing** panes |
| Nothing useful exists | Bootstrap below |

Record live ids in `WHERE-LIVE.md` after every restore.

## Worktrees (optional, fix/follow-up lanes)

Herdr can own git worktrees as workspaces — prefer this over ad-hoc `/tmp` when Joep wants panes per agent:

```bash
herdr worktree list
herdr worktree create --help
herdr worktree open --path <abs> --label <name> --no-focus
herdr worktree remove --path <abs>   # only when Joep asks / cleanup
```

For **report-only audit**, a worktree is usually unnecessary (read patch from main checkout). For **review-means-fix** after audit, open the fix worktree via Herdr so Joep can watch.

## Bootstrap (only if missing)

```bash
AUDIT=/tmp/<repo>-audit-<date>/canvas
mkdir -p "$AUDIT"

# Neighbor on conductor tab — skip if where-live pane already exists
herdr pane split <conductor-pane> --direction right --ratio 0.45 --cwd "$AUDIT" --no-focus
herdr pane rename <new-pane> where-live

herdr tab create --workspace <ws> --cwd "$AUDIT" --label map --no-focus
herdr tab create --workspace <ws> --cwd "$AUDIT" --label findings --no-focus
herdr tab create --workspace <ws> --cwd "$AUDIT" --label connections --no-focus
```

Write `WHERE-LIVE.md`, `00-map.md`, `01-findings.md`, `02-connections.md`, later `03-FINAL.md`.

## Restore after `/tmp` wipe

1. `herdr pane list` — keep existing labeled panes.
2. Rebuild markdown from brain: `reports/<date>/*thermos-audit*.md` (SSH/canonical or `brain query` / `brain deep-search`).
3. Re-render with less on **existing** pane ids (below).
4. `herdr tab focus` findings — do not recreate tabs.

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

- Creating new `map`/`findings`/`connections` tabs when labeled ones already exist
- `glow` / `less` via `bash -lc` after `send-keys q` → `qbash` paste bugs
- Closing overview tabs when re-splitting the conductor tab
- Stealing focus from Joep's other workspace without need
- Assuming `/tmp/.../canvas` survived reboot/cleanup — always check, restore from brain
