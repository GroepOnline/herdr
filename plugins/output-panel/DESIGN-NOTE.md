# Output Panel — architect synthesis

**Decision: polish-in-place.** Do not reshape the provider registry.

## Grounding

Data flow is already clean: `providers/*` → registry → `state` (select by id) →
`render` (one frame) → `tui` (keys + side effects) → `redact` on every byte.
Headless `index.js` shares the same registry. Largest module is `tui.js`
(~650 lines) but the seams (state/render/keys/format/lib) are real.

## Why not reshape

- Adding sources is already one file + one registry line.
- Friction is Operate craft (hierarchy, empty states, footer density), not
  ownership or layering.
- A rewrite would burn tokens and risk the verified PTY/exit contract.

## Operate polish backlog (when billing allows)

1. Stronger group headers / collapse affordance contrast.
2. Denser teaching empty states (already have `tried:`/`hint:` — tighten copy).
3. Footer hint truncation on narrow widths without losing the active view's verbs.
4. Optional: split follow-poll orchestration out of `tui.js` if it grows again.

## Out of scope

Core Rust UI, new npm deps, PRODUCT.md/DESIGN.md for the whole Herdr product
(impeccable scoped Operate against this plugin code is enough).

## Stack decision (Node vs OpenTUI vs Go)

**Stay on zero-dep Node ESM for this panel.** Do not rewrite output-panel in OpenTUI or Go unless Joep explicitly asks.

- **OpenTUI** ([anomalyco/opentui](https://github.com/anomalyco/opentui), pin `@opentui/core@0.4.5` unless he asks `snapshot` / a feature spike): valid for a **new** layout-heavy Herdr pane when Joep opts in. Policy + CI-only natives: skill `opentui-herdr`. Native renderer needs Bun or Node 26.4+ `--experimental-ffi` (laptop is Node 24 / no Bun today). Tree-sitter lives on OpenTUI `main`, not a separate branch.
- **Go / Bubbletea**: fine for standalone CLIs; wrong here. `go` is not on laptop `joep`, every CHEF plugin is Node, and Herdr only needs an argv command on a TTY.
- **Node**: matches `plugins/*`, edit→link→open loop, shared headless registry, already verified (130 channels, PTY restore, redact). Default for CHEF fleet / thin log browsers.
