# Research brief: `src/ghostty/` — Moshi vs mosh

> Date: 2026-08-01 · Branch: `fix/kitty-graphics-host-repaints`  
> Question (Joep): *meer optimaliseringen voor miss? of voor mosh?* while pointing at `src/ghostty/`.

## What “miss” is

There is **no product, crate, or protocol named `miss`** under `src/ghostty/` or the Herdr tree.

Evidence points to **Moshi** (iPhone mobile terminal, [getmoshi.app](https://getmoshi.app/)):

| Signal | Citation |
|---|---|
| Preview docs name Moshi as the phone client over SSH/**mosh** | `docs/next/website/src/content/docs/moshi.mdx` (title + “SSH (or mosh)” setup) |
| Website / how-to-work links Moshi | `website/src/content/docs/how-to-work.mdx`, `website/index.html` |
| Design-system session notes install Moshi | `~/design-system/docs/pi-sessions-design-system.md` (“Moshi + ChefNotify”) |
| Codebase “miss” hits are unrelated | e.g. agent confirmation *misses* in `src/pane.rs`, `DaemonDetachMissing` in `src/remote/unix.rs` — not a client product |

Dutch speech/typo: **Moshi → miss** is the reading that matches the paired alternative **mosh**.

## What mosh is (in this product)

**mosh** is a *transport option* for reaching a Herdr host (UDP remote shell). Herdr does **not** implement the mosh protocol. Docs treat it as interchangeable with SSH for phone reachability:

```text
Host reachability | SSH / mosh / Tailscale
```

— `docs/next/website/src/content/docs/moshi.mdx` (Why Herdr + Moshi table).

## What `src/ghostty/` actually owns

`src/ghostty/{mod.rs,bindings.rs}` is the **libghostty-vt FFI** surface: VT parsing, Kitty graphics placements/fingerprints, keyboard state, batched vendor reads. Callers that matter for phone/remote UX:

- `src/kitty_graphics.rs` — host Kitty image upload/placement cache
- `src/client/mod.rs` — attach client blit + graphics write path
- `src/ui/mobile.rs` — narrow-width (Moshi-sized) layout
- `src/server/headless.rs` + `pane_graphics.rs` — streaming graphics to attached clients

Optimizing “for mosh” inside ghostty would mean guessing at UDP loss — Herdr never sees mosh frames. Optimizing “for Moshi” means: reliable Kitty images over attach, cheap repaints, and mobile layout — all of which sit on ghostty + kitty_graphics + client.

## Recommendation

**Optimize for Moshi (the client UX), not for mosh (the transport).**

1. **Primary:** keep Kitty graphics correct across host repaints / focus / resize (this PR ports upstream `36de78dd`).
2. **Next ghostty slice:** more batched FFI (`ghostty_*_get` multi-field helpers already documented in `bindings.rs`) and fingerprint cache hit-rate — helps every attach client, Moshi included.
3. **Do not** invest in mosh-specific code in Herdr; Tailscale + SSH (or user-chosen mosh) stays outside the binary.
4. **Mobile follow-ups** stay in `src/ui/mobile.rs` + settings IA (see `docs/CHEF-SETTINGS-UI-DESIGN.md` rewrite plan), not in the VT FFI.

## Related image bug (this PR)

`docs/next/UPSTREAM_SWEEP.md` tracked upstream `36de78dd` — *preserve kitty graphics during host repaints* (refs upstream #1628) as **open**. Clearing the host surface on “full redraw” deleted live Kitty images (visible after focus gain / forced repaint). This branch ports that fix and marks the sweep row **ported**.
