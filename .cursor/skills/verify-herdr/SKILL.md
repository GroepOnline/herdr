---
name: verify-herdr
description: "Drive Herdr like a user — headless CLI e2e, mouse-first TUI (settings/modals/pane splits), optional future sandbox. Use when proving pane/workspace/settings behavior, smoke-testing a binary, or capturing desktop evidence without cargo builds on Cloud VMs."
---

# Verify Herdr

Herdr is a terminal workspace manager: a **headless server** owns panes/PTY state; the **CLI** talks over a Unix socket; the **TUI** is one mouse-first client.

Two proof tracks:

| Track | When | Entry |
| --- | --- | --- |
| **CLI / headless** | CI, Cloud without desktop, fast gates | `scripts/e2e.sh` |
| **Mouse-first TUI** | Settings, modals, layout UX, demos | `features/settings-and-modals.md` + `scripts/tui-mouse-checklist.sh` |
| **Sandbox** (later) | Disposable display/fs policy | `features/sandbox.md` (`HERDR_VERIFY_SANDBOX` stub) |

On Cursor Cloud VMs, local `cargo` / `just check|test|lint` are blocked. Use a release or CI smoke binary. Never drive the user’s installed stable server — always isolate with a dedicated `HOME`.

## Launch

### Binary

```bash
export HERDR_BIN=/opt/herdr/herdr-linux-x86_64
# or:
.cursor/skills/verify-herdr/scripts/download-binary.sh
source .cursor/skills/verify-herdr/scripts/env.sh
```

Refresh Cloud env + Cursor standards:

```bash
bash .cursor/scripts/cloud-install.sh
```

### Headless server

```bash
source .cursor/skills/verify-herdr/scripts/env.sh
.cursor/skills/verify-herdr/scripts/launch-server.sh
.cursor/skills/verify-herdr/scripts/doctor.sh
```

Teardown: `scripts/cleanup.sh` (keeps evidence).

### Interactive TUI

Needs a real TTY (Cloud: xfce4 on `:1`). Dark terminal preferred.

```bash
env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH \
  HOME="$HERDR_HOME" "$HERDR_BIN"
```

## CLI drive

Wrapper:

```bash
.cursor/skills/verify-herdr/scripts/cli.sh <args…>
```

### Full e2e (preferred CLI)

```bash
.cursor/skills/verify-herdr/scripts/e2e.sh
```

Covers: server lifecycle → doctor → workspace create → pane run/read → send-text/keys/wait → split → tab → **config check / api snapshot** → final status.

### Minimal smoke

```bash
.cursor/skills/verify-herdr/scripts/smoke-pane-run-read.sh
```

## Mouse-first TUI drive

Herdr is mouse-first. Prove settings and layout with clicks, not only prefix keys.

### Settings + modals

1. Click sidebar **menu** → **settings** (proven path on 0.7.7). Fallback: `Ctrl+b` then `s`.
2. Click through sections: appearance, layout, input, terminal, notifications, agents, plugins, updates, advanced.
3. Dismiss: close / Esc / outside (body chrome = no-op).
4. Details: `features/settings-and-modals.md`.

### Extra panes + commands

1. Right-click pane → **Split right** / **Split down**.
2. Click panes to focus (active border).
3. Run markers per pane (`echo …`, `herdr --version`, `pwd`).
4. Details: `features/pane-layout-mouse.md`.

### Checklist / artifact gate

```bash
.cursor/skills/verify-herdr/scripts/tui-mouse-checklist.sh
.cursor/skills/verify-herdr/scripts/tui-mouse-checklist.sh --verify
```

`--verify` expects demo artifacts under `/opt/cursor/artifacts/` (mp4 + key screenshots).

## Sandbox (planned)

Isolated `HOME` is required today. Nested display / policy sandbox is **not** wired yet — see `features/sandbox.md`. When implemented, `HERDR_VERIFY_SANDBOX=1` should wrap both CLI e2e and mouse checklists.

## Evidence

| Kind | Path |
| --- | --- |
| CLI runs | `/opt/cursor/artifacts/herdr-verify/<run-id>/` |
| Mouse demos | `/opt/cursor/artifacts/herdr-mouse-*.mp4`, `screenshots/herdr-mouse-*.webp` |

Proof standards:

- Real CLI/user paths (`workspace create`, `pane run`, menu → settings), not internal setters.
- Capture **action + result** (pane text, settings screenshot), not only exit codes.
- No mocks for the PTY path.

## Cleanup

```bash
.cursor/skills/verify-herdr/scripts/cleanup.sh
```

Never deletes evidence; never `pkill herdr` by name.

## Helpers

| Script | Role |
| --- | --- |
| `download-binary.sh` | Release → CI smoke |
| `env.sh` | Binary + isolated `HOME` + evidence dir |
| `launch-server.sh` / `doctor.sh` / `cli.sh` | Headless lifecycle |
| `e2e.sh` | Full CLI feature-map proof |
| `smoke-pane-run-read.sh` | Narrow smoke |
| `tui-mouse-checklist.sh` | Mouse drive checklist + `--verify` |
| `cleanup.sh` | Tear down instance |

## Feature map

See `features/README.md`.
