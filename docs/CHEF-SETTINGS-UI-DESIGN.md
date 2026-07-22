# CHEF Settings UI Design

> Historical wishlist + current shipping target. The 2026-07-14 tabbed modal was
> replaced by the left-nav customization shell on branch
> `cursor/settings-ui-rework-a115`.

## Settings Overlay (Prefix + S)

Accessible via the settings keybinding (default `prefix+s`) — modal overlay with
left navigation, search, and scrollable section content.

### Layout

```
┌─ customize herdr ───────────────────────────────────────────────────┐
│ / search…                                           [Apply] [Close] │
│ ┌ Left nav ┐  ┌ Content ─────────────────────────────────────────┐  │
│ │ appearance │  Section title · short help                        │  │
│ │ layout     │  toggles / choices / live previews                 │  │
│ │ input      │                                                    │  │
│ │ terminal   │                                                    │  │
│ │ notify     │                                                    │  │
│ │ agents ●   │                                                    │  │
│ │ updates    │                                                    │  │
│ │ advanced   │                                                    │  │
│ └────────────┘  └────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Implementation

- Module: `src/ui/settings/` (`layout`, `rows`, `sections`, `spinner`)
- Input/actions: `src/app/input/settings.rs` + `src/app/config_io.rs`
- Reads/writes `~/.config/herdr/config.toml` and hot-reloads via
  `apply_config_from_disk`
- Shared `SettingsLayout` drives render and mouse hit-testing
- Animations: `SettingsState.preview_tick` while `Mode::Settings`

### Sections

| Section | Options |
|---|---|
| Appearance | themes, host auto-switch, categorized spinner picker |
| Layout | pane chrome, sidebar collapse/sort, pane templates |
| Input | mouse/copy/focus redraw, confirms/prompts, host cursor, keybind help CTA |
| Terminal | default shell, shell mode, new cwd, scrollback presets |
| Notifications | sound, toast delivery/delay/position, clipboard toasts |
| Agents | resume-on-restore, integration install/status |
| Updates | channel, version check, manifest check |
| Advanced | experiments, kitty graphics, nested, remote SSH, clipboard history, config path |

### Non-goals (v1)

- Full in-modal plugin marketplace browser
- Full keybind editor (use existing Keybind Help)
- Sidebar token DSL composer
- Workspace TOML templates
