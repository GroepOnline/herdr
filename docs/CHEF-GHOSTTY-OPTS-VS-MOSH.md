# Research brief: `src/ghostty/` — maybe more opts, or mosh?

> Date: 2026-08-01 · Branch: `fix/kitty-graphics-host-repaints`  
> Question (Joep): *meer optimaliseringen voor miss? of voor mosh?* while pointing at `src/ghostty/`.

## Correction (etymology)

**`miss` here is Dutch `misschien` (= maybe), not Moshi.**

Joep asked, roughly: *more optimizations, maybe? or for mosh?* — a choice between further general ghostty/graphics work versus investing in **mosh** as a transport angle.

An earlier draft of this brief falsely read `miss` as **Moshi** (the iPhone terminal at [getmoshi.app](https://getmoshi.app/)). That product exists in Herdr docs as a separate phone client surface, but it was **not** what Joep named in this question.

## What mosh is (in this product)

**mosh** is an optional *transport* for reaching a host (UDP remote shell). Herdr does **not** implement the mosh protocol (no `mosh` crate/module under `src/`). Product docs sometimes list it alongside SSH/Tailscale for phone reachability; that stays outside the Herdr binary.

## What `src/ghostty/` actually owns

`src/ghostty/{mod.rs,bindings.rs}` is the **libghostty-vt FFI** surface: VT parsing, Kitty graphics placements/fingerprints, keyboard state, batched vendor reads. Callers that matter for attach / image UX:

- `src/kitty_graphics.rs` — host Kitty image upload/placement cache
- `src/client/mod.rs` — attach client blit + graphics write path (`request_repaint`, received-image tracking)
- `src/ui/mobile.rs` — narrow-width layout (helps any small client, including phone apps)
- `src/server/headless.rs` + `pane_graphics.rs` — streaming graphics to attached clients

Optimizing “for mosh” inside ghostty would mean guessing at UDP loss — Herdr never sees mosh frames. Further ghostty/graphics work helps **every** attach client (local Ghostty, SSH, phone apps such as Moshi if used).

## Recommendation (priority)

1. **Primary (this PR):** keep Kitty graphics correct across host repaints / focus / resize (ports upstream `36de78dd`). Valid regardless of the misschien/mosh framing.
2. **Next general opts:** more batched FFI (`ghostty_*_get` multi-field helpers already documented in `bindings.rs`) and fingerprint cache hit-rate — all attach clients benefit.
3. **Do not** invest in mosh-specific code in Herdr; Tailscale + SSH (or user-chosen mosh) stays outside the binary.
4. **Settings rewrite** (`docs/CHEF-SETTINGS-UI-DESIGN.md`): P0 blurbs/truncation ship with this PR; P1+ IA work is a separate track after graphics correctness.
5. **Moshi** remains a documented phone client (`docs/next/website/.../moshi.mdx`) — useful product surface, but not the parse of this question.

## Related image bug (this PR)

`docs/next/UPSTREAM_SWEEP.md` tracked upstream `36de78dd` — *preserve kitty graphics during host repaints* (refs upstream #1628) as **open**. Clearing the host surface on “full redraw” deleted live Kitty images (visible after focus gain / forced repaint). This branch ports that fix and marks the sweep row **ported**.
