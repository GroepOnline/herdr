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
│ │ plugins    │                                                    │  │
│ │ updates    │                                                    │  │
│ │ advanced   │                                                    │  │
│ └────────────┘  └────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Implementation

- Module: `src/ui/settings/` (`catalog`, `layout`, `rows`, `sections`, `spinner`)
- Typed catalog: every row has a `SettingsItemId` (no opaque `usize` payloads)
- Input/actions: `src/app/input/settings.rs` + `src/app/config_io.rs`
- Reads/writes `~/.config/herdr/config.toml` and hot-reloads via
  `apply_config_from_disk`
- Shared `SettingsLayout` drives render and mouse hit-testing
- Animations: `SettingsState.preview_tick` while `Mode::Settings`
- CHEF personalization (Fleet/Plugins kitchen-sink tabs, `com.chefgroep.*`,
  `ENG-` branding) lives in private `OnlineChefGroep/herdr-plugins`, not in
  core settings. Core exposes a native **Plugins** section with installed
  plugin toggles and a curated CHEF catalog install list, plus a generic Advanced
  **fleet ops bar** toggle and a cached, plugin-id-agnostic `fleet_ops.json`
  merge off the render path.

### Sections

| Section | Options |
|---|---|
| Appearance | themes, host auto-switch, spinner hero + categorized picker |
| Layout | pane chrome, sidebar collapse/sort, compact templates (incl. monitoring) |
| Input | mouse/copy/focus redraw, confirms/prompts, host cursor, keybind help CTA |
| Terminal | default shell, shell mode, new cwd, scrollback presets |
| Notifications | sound, toast delivery/delay/position, clipboard toasts |
| Agents | resume-on-restore, integration install/status |
| Plugins | installed on/off + one-step curated installs (friendly names, no CLI dump) |
| Updates | channel, version check, manifest check |
| Advanced | experiments, fleet ops bar, kitty/nested/SSH, clipboard history, config path |

### Non-goals (v1)

- Full public marketplace browser (curated installs; private pack in `herdr-plugins`)
- Full keybind editor (use existing Keybind Help)
- Sidebar token DSL composer
- Workspace TOML templates
- ASCII wireframe template cards (removed — plain apply rows)
