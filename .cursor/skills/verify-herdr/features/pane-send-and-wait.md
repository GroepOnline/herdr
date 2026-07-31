# Pane send-text / wait-output

## What

Type into a live pane (send text + Enter) and wait until matching output appears.

## Reach

Server up, workspace focused, pane id known (`pane list`).

## Drive

```bash
# Resolve PANE from workspace create JSON (or pane list), never assume w1:p1.
.cursor/skills/verify-herdr/scripts/cli.sh pane send-text "$PANE" 'echo send-wait-ok'
# If send-text does not append newline, send Enter:
.cursor/skills/verify-herdr/scripts/cli.sh pane send-keys "$PANE" Enter
.cursor/skills/verify-herdr/scripts/cli.sh pane wait-output "$PANE" --match 'send-wait-ok' --timeout 5000
.cursor/skills/verify-herdr/scripts/cli.sh pane read "$PANE" --source recent --format text
```

Confirm flag names with `pane send-text --help` / `pane send-keys --help` on `$HERDR_BIN` — they can differ slightly across versions.

## Prove

- `wait-output` succeeds within timeout.
- Recent pane text includes `send-wait-ok`.
- Prefer this over `pane run` when proving interactive input encoding.
