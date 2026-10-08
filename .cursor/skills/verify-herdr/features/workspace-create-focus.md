# Workspace create + focus

## What

Create a workspace in a chosen cwd and focus it so panes target that workspace.

## Reach

Headless server up (`scripts/launch-server.sh` + `doctor.sh`).

## Drive

```bash
.cursor/skills/verify-herdr/scripts/cli.sh workspace create --cwd "$HERDR_HOME" --label verify --focus
.cursor/skills/verify-herdr/scripts/cli.sh workspace list
.cursor/skills/verify-herdr/scripts/cli.sh pane list
```

## Prove

- `workspace list` shows the new workspace (label/cwd as configured).
- `pane list` shows at least one pane in that workspace.
- Save list output under `$HERDR_VERIFY_EVIDENCE`.
