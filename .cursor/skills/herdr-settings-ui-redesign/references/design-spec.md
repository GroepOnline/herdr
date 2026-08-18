# Settings design spec (v1)

## Shell

- Wider modal (~88–100 cols), taller content, left nav + scrollable body.
- Search/filter over setting labels.
- Footer: Done/Close; primary actions per section when needed (Apply theme, Install integrations).
- Animations via `preview_tick` while settings is open (runtime-owned, never in render).

## Sections

| Section | Surfaces |
|---|---|
| Theme | themes, auto dark/light, live preview |
| UI | spinner categories/pages, status indicators, pane chrome, sidebar, input |
| Sound | sound alerts, toast delivery/position/delay, clipboard toasts |
| System | shell, updates, experiments, kitty/nested/SSH, worktrees, config path |
| Templates | pane layout templates |
| Integrations | resume-on-restore, agent CLI install/status, installed plugins, catalog |

## Non-goals v1

- Full in-modal plugin marketplace
- Full keybind editor
- Sidebar token DSL composer
- Workspace TOML templates
