# Architecture map

| File | Owns |
|---|---|
| `src/ui/settings.rs` (or `src/ui/settings/`) | Shell chrome, layout rects, section render, hit helpers |
| `src/app/input/settings.rs` | `SettingsAction`, key/mouse routing, open/cancel/apply |
| `src/app/state.rs` | `SettingsSection`, `SettingsState`, selection/search/focus |
| `src/app/config_io.rs` | Persist toggles/choices to config.toml |
| `src/app/runtime.rs` | `preview_tick` + animation timer while settings open |
| `src/ui/widgets.rs` | Shared modal primitives |
| `src/config/model.rs` | Config field types |

## Parallel seams

1. Shell + `SettingsLayout` + nav/search hit-tests
2. Appearance / Layout / Notifications + save helpers
3. Input / Terminal / Updates / Agents / Advanced + save helpers
