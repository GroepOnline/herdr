# Upstream sweep (ogulcancelik/herdr)

Track useful upstream fixes that are not yet in OnlineChefGroep/herdr.
Do **not** port Hermes-related changes.

Merge-base reference: see `git merge-base origin/main upstream/master`.

## MUST-PORT (no Hermes)

| Upstream | Title | Status |
|---|---|---|
| `ef85fa0c` | keep reading while pty input blocked | open |
| `c0fb777e` | truncated pane reads | open |
| `08640bb3` | remote path discovery `/bin/sh` fallback | **ported** |
| `169a4fd9` | XTWINOPS size queries | open |
| `58e812b2` | url click gestures | open |
| `2a20e90a` | physical escape on Windows | open |
| `e7fc85bf` | kitty printable key releases | open (protocol care) |
| `36de78dd` | kitty graphics host repaints | open |
| `8afd52ae` | bundle modern ConPTY | open (packaging) |
| `7e821ba7` | OMP `PI_CONFIG_DIR` override | **ported** |
| `e608a751` | known-agent exit process-owned | open (strip Hermes files) |

## SHOULD-PORT

| Upstream | Title | Status |
|---|---|---|
| `0065a0ef` | pi worktree background (no focus) | **ported** |
| `d4e0dd3d` | workspace git discovery perf | open |
| `b33e5f97` | copy retained mouse selections | open |
| `38d2b078` | filter keybind help | open |
| `e9b22084` | grok CLI native restore | open |

## SKIP

- Hermes lifecycle harden
- Apache relicense
- Contributor approve chores
- Host palette (already on CHEF via #63)
