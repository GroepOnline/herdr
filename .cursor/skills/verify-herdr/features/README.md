# Herdr verification feature map

User-facing features to prove with `.cursor/skills/verify-herdr/`. Drive the **real** headless server + CLI (or mouse-first TUI) path named in each file. Prefer these entry points over one-off shortcuts.

## CLI / headless

| Feature | File | Surface |
| --- | --- | --- |
| Headless server lifecycle | `server-lifecycle.md` | CLI `server` / `status` |
| Workspace create + focus | `workspace-create-focus.md` | CLI `workspace` |
| Pane run + read | `pane-run-and-read.md` | CLI `pane` (primary smoke) |
| Pane send-text / wait-output | `pane-send-and-wait.md` | CLI `pane` |
| Status JSON doctor | `status-json.md` | CLI `status --json` |
| Pane layout (CLI split) | `pane-layout-mouse.md` | CLI `pane split` + TUI |

**Full CLI e2e:** `../scripts/e2e.sh`

**Narrow smoke:** `pane-run-and-read.md` via `../scripts/smoke-pane-run-read.sh`.

## Mouse-first TUI

| Feature | File | Surface |
| --- | --- | --- |
| Settings + modals | `settings-and-modals.md` | sidebar menu → settings, sections, dismiss |
| Pane layout (mouse) | `pane-layout-mouse.md` | context-menu split, focus clicks |
| Checklist helper | `../scripts/tui-mouse-checklist.sh` | print / `--verify` artifacts |

## Sandbox (later)

| Feature | File | Surface |
| --- | --- | --- |
| Sandbox launcher stub | `sandbox.md` | `HERDR_VERIFY_SANDBOX` (not wired yet) |
