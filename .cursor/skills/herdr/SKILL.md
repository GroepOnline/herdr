---
name: herdr
description: Develop and change the herdr terminal agent runtime (Rust TUI, session server, socket API, detection manifests, integrations). Use when editing herdr src/, running just check/test, working on agent detection, protocol/API, or following AGENTS.md contribution rules.
---

# Herdr development

## Commands

Prefer `just` (see root `justfile`):

```bash
just lint    # fmt check + clippy -D warnings
just test    # nextest + python maintenance + bun suites
just check   # ci + windows-lint + maintenance tests
just test-one <filter>
cargo build  # or cargo run -- <cmd>
```

Cursor Cloud: if local `cargo`/`zig`/`just test|check|lint` is blocked by hooks, validate via GitHub Actions (`gh pr checks`).

## Invariants

From `AGENTS.md` — do not violate:

1. `AppState` pure; no PTY required for state tests
2. `render()` never mutates state
3. Platform code only under `src/platform/`
4. Detection reads snapshots only; evidence-based manifests
5. Server owns runtime facts; TUI owns presentation only

## Tests

- Unit tests beside code (`#[cfg(test)]`)
- `AppState::test_new()` / `Workspace::test_new()`
- Identity refactors: `assert_invariants_for_test()` + adversarial helpers

## Commits

Lowercase conventional commits. Issue body line: `refs #<n>` (not `fixes`/`closes`). No AI co-author trailers.

## Docs

- Stable: `website/src/content/docs/`
- Unreleased: `docs/next/website/src/content/docs/`
- Do not hand-edit `website/latest.json` / preview channel files unless release work

## Source run

```bash
env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH cargo run -- <command>
```

Headless: `./target/debug/herdr server` + CLI/socket. TUI needs a real TTY.

## Detection loop

1. `herdr agent read <pane> --source detection --format text` (+ `ansi` if needed)
2. Edit `src/detect/manifests/<agent>.toml`
3. Optional override + `herdr server reload-agent-manifests`
4. Remove override when done

## CHEF fleet

For Linear/GitHub/UDO/Kater, Fleet Ops Bar, and `com.chefgroep.*` plugins, use the **chef-fleet** skill (`.cursor/skills/chef-fleet/SKILL.md`). UI dismiss rules and bar presentation live in **herdr-ui**.

## External contributors

If GitHub account is not `OnlineChefGroep`, follow `CONTRIBUTING.md`. Agents must not open GitHub issues on a human’s behalf.
