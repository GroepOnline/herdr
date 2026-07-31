# Status JSON doctor

## What

Inspect client/server runtime status as structured JSON before and after drives.

## Reach

Any isolated `HOME` (server optional). Binary must be on `HERDR_BIN`.

## Drive

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/cli.sh status --json | tee "$HERDR_VERIFY_EVIDENCE/status.json"
.cursor/skills/verify-herdr/scripts/doctor.sh   # requires running server
```

## Prove

- JSON parses and includes `client.version`, `server.running`, `server.socket`.
- Socket path is under `$HERDR_HOME/.config/herdr/` (isolation check).
- Doctor script exits 0 only when the server is up and the recorded PID lives.
