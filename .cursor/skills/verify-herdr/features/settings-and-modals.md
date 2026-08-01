# Settings + modals (mouse-first TUI)

## What

Prove the settings overlay and related chrome the way a user does: **mouse hits**, not only prefix keys. Also covers dismiss contract and section navigation.

## Reach

Interactive TUI on a real TTY (Cloud: xfce4 `:1`). Isolated `HOME`. Binary on `PATH` or `$HERDR_BIN`.

Settings open default: **prefix + s** (`Ctrl+b` then `s`). Prefer mouse via sidebar **menu** / context menu when the build exposes it.

## Drive (mouse-first)

1. Launch TUI:

```bash
env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH \
  HOME="$HERDR_HOME" "$HERDR_BIN"
```

2. Open Settings:
   - **Mouse:** click sidebar `menu` → Settings (or equivalent), or a settings entry in a context menu.
   - **Fallback keyboard:** `Ctrl+b` then `s`.
3. Click through sections with the mouse (whatever the build lists): Theme / Appearance / Sound / Toasts / Plugins / Fleet / Integrations / Spinner, etc.
4. Dismiss: click **close**, or **Esc**, or **outside** the popup (cancel). Body chrome clicks are no-ops — do not treat them as Cancel.
5. Capture screenshots of open settings + at least one other section.

## Prove

- Settings overlay is visible (title/sections/rows).
- At least two sections navigated by mouse click.
- Dismiss returns to the normal workspace without a stuck modal.
- Evidence: screenshots / short mp4 under `/opt/cursor/artifacts/`.

## CLI cousins (no modal)

Settings UI is TUI presentation. Config validation / reload stay CLI:

```bash
.cursor/skills/verify-herdr/scripts/cli.sh config check
.cursor/skills/verify-herdr/scripts/cli.sh server reload-config
.cursor/skills/verify-herdr/scripts/cli.sh api snapshot
```

## Sandbox note

When `HERDR_VERIFY_SANDBOX=1` is wired (see `features/sandbox.md`), run this TUI path inside the sandbox display/session instead of the shared desktop.
