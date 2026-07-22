# Settings design spec (v1)

## Shell

- Wider modal (~88–100 cols), taller content, left nav + scrollable body.
- Search/filter over setting labels.
- Footer: Done/Close; primary actions per section when needed (Apply theme, Install integrations).
- Animations via `preview_tick` while settings is open (runtime-owned, never in render).

## Sections

| Section | Surfaces |
|---|---|
| Appearance | themes, auto dark/light, spinner categories/pages |
| Layout | pane chrome, sidebar collapse/sort, templates |
| Input | mouse, host cursor, confirms/prompts; link to Keybind Help |
| Terminal | shell, shell mode, new cwd, scrollback |
| Notifications | sound, toast delivery/position/delay, clipboard toasts |
| Agents | resume-on-restore, integrations install/status |
| Updates | channel, version/manifest checks |
| Advanced | experiments, kitty graphics, nested, worktrees dir, remote SSH, open/reload config CTA |

## Non-goals v1

- Full in-modal plugin marketplace
- Full keybind editor
- Sidebar token DSL composer
- Workspace TOML templates
