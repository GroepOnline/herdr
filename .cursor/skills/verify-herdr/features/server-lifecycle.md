# Headless server lifecycle

## What

Start a persistent headless server, confirm it is running, stop it cleanly.

## Reach

Isolated `HOME` + smoke binary (`scripts/env.sh`).

## Drive

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/launch-server.sh
.cursor/skills/verify-herdr/scripts/doctor.sh
.cursor/skills/verify-herdr/scripts/cli.sh status --json
.cursor/skills/verify-herdr/scripts/cli.sh server stop
```

## Prove

- After launch: `status --json` has `"running":true` and socket under `$HERDR_HOME`.
- After stop: `status --json` has `"running":false` (or socket gone) and the recorded PID is dead.
- Do not prove this by killing random `herdr` processes on the machine.
