---
name: herdr-ui
description: Polish and harden the herdr mouse-first TUI — premium consistent overlays, sidebar/tabs/dialogs, pure render, and fix memory leaks or UI errors. Use when editing src/ui/, src/ui.rs, app/input modals/overlays/sidebar/mouse, or when the user asks for premium UI, visual polish, jank, flicker, leaks, or UI bugs.
---

# Herdr UI (premium + leak-free)

## Goal

Ship TUI changes that feel **premium and consistent**, and actively remove **leaks, stale state, and interaction errors**.

## Where to edit

| Surface | Path |
|---------|------|
| Draw | `src/ui.rs`, `src/ui/` |
| Input | `src/app/input/` (`modal`, `overlays`, `sidebar`, `mouse`, …) |
| Shared widgets/text | `src/ui/widgets.rs`, `src/ui/text.rs` |

Do **not** put sidebar/modal/selection presentation on the server protocol.

## Premium checklist

Copy and track:

```
UI Progress:
- [ ] Reused existing modal/overlay/sidebar pattern (no one-off screen)
- [ ] render() stays pure (no state mutation)
- [ ] Hit-test rects match drawn geometry
- [ ] Focus/hover/active states clear after resize
- [ ] Keyboard path complete (mouse-first, not mouse-only)
- [ ] Narrow/mobile path considered (src/ui/mobile.rs)
- [ ] Footer/key hints accurate
- [ ] Leak/retention review done (see below)
```

## Hard rules

1. **Reuse UI language** — onboarding, settings, dialogs, navigator, menus, release notes. Same close affordances and structure.
2. **`render()` pure** — mutations only in `compute_view()` / actions / input.
3. **Client-only presentation** — colors, tokens, selection, mouse, viewport scroll for UI chrome.
4. **No production `unwrap()`** on UI paths.

## Leak & error pass

Always scan touched paths for:

- Unbounded per-frame / per-event growth (lists, maps, toast/overlay stacks)
- Heavy `clone()` in hot render/input loops
- State not cleared on overlay dismiss or pane/workspace remove
- Stale drag/scrollbar/focus after close
- Hit areas desynced from `render_*`
- Double-modal input capture
- Scroll metrics off-by-one

Fix leaks with clear-on-dismiss, drop-on-remove, and bounded buffers — not by papering over symptoms.

## Method

1. Classify: polish vs interaction bug vs leak/perf (fix correctness/leaks first when mixed).
2. Open matching `render_*` + input handler + `*_rect` helpers together.
3. Prefer extending `dialogs`, `menus`, `navigator`, `settings`, `sidebar`, `widgets`.
4. Add pure geometry/state unit tests where possible (no PTY).
5. Validate: `just lint` / `just test-one <filter>`; if cargo blocked in Cloud VM, use CI.

## Review output format

- **Critical** — crash, dead input, unbounded growth, mutate-in-render
- **Should fix** — pattern break, bad hit targets, flicker, stale overlay
- **Polish** — hierarchy, spacing, hints aligned with existing surfaces

Name the existing module to reuse for each fix.
