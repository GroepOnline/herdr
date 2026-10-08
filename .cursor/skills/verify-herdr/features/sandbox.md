# Sandbox (planned)

## What

Run verify-herdr drives inside an isolated sandbox so CLI + TUI proofs do not touch the shared Cloud desktop or a developer’s stable Herdr install.

## Status

**Stub.** Scripts today use an isolated `HOME` + cleared socket env. Full sandbox (nested display, network/fs policy, disposable profile) is intentionally not wired yet.

## Contract (when enabled)

| Knob | Meaning |
| --- | --- |
| `HERDR_VERIFY_SANDBOX=1` | Prefer sandbox launcher over bare desktop/tmux |
| `HERDR_VERIFY_SANDBOX_DISPLAY` | Optional nested X/Wayland display (e.g. `:99`) |
| `HERDR_VERIFY_SANDBOX_ROOT` | Scratch root for home, sockets, evidence mirrors |

Expected launcher shape (not implemented yet):

```bash
# future
.cursor/skills/verify-herdr/scripts/sandbox-run.sh -- \
  .cursor/skills/verify-herdr/scripts/e2e.sh
```

## Current isolation (use this now)

Already enough for Cloud CLI proofs:

```bash
source .cursor/skills/verify-herdr/scripts/env.sh   # fresh HERDR_HOME under /tmp
.cursor/skills/verify-herdr/scripts/launch-server.sh
.cursor/skills/verify-herdr/scripts/cli.sh …
.cursor/skills/verify-herdr/scripts/cleanup.sh
```

TUI mouse demos still need a real desktop (`DISPLAY=:1`); document evidence paths, do not reuse the user’s `~/.config/herdr`.

## Prove (later)

- Sandbox start/stop is idempotent and does not kill non-sandbox herdr PIDs.
- CLI e2e + optional mouse checklist both pass with `HERDR_VERIFY_SANDBOX=1`.
- Evidence still lands under `/opt/cursor/artifacts/herdr-verify/<run-id>/`.
